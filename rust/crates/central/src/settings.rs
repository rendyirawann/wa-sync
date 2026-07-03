use std::collections::HashMap;
use std::path::PathBuf;

use axum::{
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::{auth, rbac::CurrentUser, view, AppState};

/// Default settings (fallback `?? '...'` di Blade).
fn defaults() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("site_name".into(), "StarterTemp".into());
    m.insert("site_logo".into(), "base-logo.png".into());
    m.insert("site_font".into(), "Plus Jakarta Sans".into());
    m.insert("maintenance_mode".into(), "0".into());
    for p in ["google", "facebook", "github", "linkedin"] {
        m.insert(format!("social_{p}_enabled"), "0".into());
        m.insert(format!("social_{p}_client_id"), String::new());
        m.insert(format!("social_{p}_client_secret"), String::new());
    }
    // Billing — Midtrans Snap (lihat midtrans.rs / billing.rs).
    m.insert("midtrans_enabled".into(), "0".into());
    m.insert("midtrans_is_production".into(), "0".into());
    m.insert("midtrans_server_key".into(), String::new());
    m.insert("midtrans_client_key".into(), String::new());
    m
}

/// Semua setting (key→value), default di-overlay DB. ≈ `Setting::allCached()`.
pub async fn all(pool: &PgPool) -> HashMap<String, String> {
    let mut map = defaults();
    let rows: Vec<(String, Option<String>)> = sqlx::query_as("SELECT key, value FROM settings")
        .fetch_all(pool).await.unwrap_or_default();
    for (k, v) in rows {
        map.insert(k, v.unwrap_or_default());
    }
    map
}

pub fn get<'a>(map: &'a HashMap<String, String>, key: &str, default: &'a str) -> &'a str {
    map.get(key).map(String::as_str).filter(|s| !s.is_empty()).unwrap_or(default)
}

/// Upsert satu setting.
pub async fn set(pool: &PgPool, key: &str, value: &str) {
    let _ = sqlx::query(
        "INSERT INTO settings (key, value, created_at, updated_at) VALUES ($1, $2, now(), now()) \
         ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = now()",
    ).bind(key).bind(value).execute(pool).await;
}

fn logos_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../public/assets/media/logos")
}

fn font_list() -> Vec<Value> {
    [
        ("Plus Jakarta Sans", "Plus+Jakarta+Sans:wght@300;400;500;600;700;800"),
        ("Inter", "Inter:wght@300;400;500;600;700"),
        ("Outfit", "Outfit:wght@300;400;500;600;700;800"),
        ("Poppins", "Poppins:wght@300;400;500;600;700"),
        ("DM Sans", "DM+Sans:wght@300;400;500;600;700"),
        ("Nunito", "Nunito:wght@300;400;500;600;700;800"),
        ("Figtree", "Figtree:wght@300;400;500;600;700;800"),
        ("Manrope", "Manrope:wght@300;400;500;600;700;800"),
    ]
    .iter()
    .map(|(name, query)| json!({ "name": name, "query": query }))
    .collect()
}

// ---------- handlers ----------

pub async fn index(user: CurrentUser, session: tower_sessions::Session, State(state): State<AppState>) -> Html<String> {
    let s = all(&state.pool).await;
    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://127.0.0.1:8090".into());
    let mut ctx = view::base_context(&state, &session, &user, "settings").await;
    ctx.insert("settings", &s);
    ctx.insert("app_url", &app_url);
    ctx.insert("fonts", &font_list());
    match state.tera.render("backend/settings/index.html", &ctx) {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

pub async fn update(session: tower_sessions::Session, State(state): State<AppState>, headers: HeaderMap, mut mp: Multipart) -> Response {
    let mut fields: HashMap<String, String> = HashMap::new();
    let mut logo: Option<(String, Vec<u8>)> = None;
    while let Ok(Some(field)) = mp.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "site_logo" {
            let fname = field.file_name().unwrap_or("").to_string();
            let bytes = field.bytes().await.map(|b| b.to_vec()).unwrap_or_default();
            if !fname.is_empty() && !bytes.is_empty() { logo = Some((fname, bytes)); }
        } else if let Ok(v) = field.text().await {
            fields.insert(name, v);
        }
    }

    let token = fields.get("_token").cloned().unwrap_or_else(|| headers.get("x-csrf-token").and_then(|v| v.to_str().ok()).unwrap_or("").to_string());
    if !auth::verify_csrf(&session, &token).await {
        return (StatusCode::from_u16(419).unwrap(), Html("<h1>419 Page Expired</h1>")).into_response();
    }

    let pool = &state.pool;
    if let Some(v) = fields.get("site_name") { set(pool, "site_name", v).await; }
    if let Some(v) = fields.get("site_font") { set(pool, "site_font", v).await; }
    set(pool, "maintenance_mode", if fields.contains_key("maintenance_mode") { "1" } else { "0" }).await;

    // Logo upload → public/assets/media/logos
    if let Some((fname, bytes)) = &logo {
        let ext = fname.rsplit('.').next().unwrap_or("").to_lowercase();
        if ["png", "jpg", "jpeg", "svg", "webp"].contains(&ext.as_str()) && bytes.len() <= 2 * 1024 * 1024 {
            let dir = logos_dir();
            let _ = std::fs::create_dir_all(&dir);
            let filename = format!("site-logo-{}.{}", chrono::Utc::now().timestamp(), ext);
            if std::fs::write(dir.join(&filename), bytes).is_ok() {
                let old = all(pool).await.get("site_logo").cloned().unwrap_or_default();
                if !old.is_empty() && old != "base-logo.png" {
                    let _ = std::fs::remove_file(dir.join(&old));
                }
                set(pool, "site_logo", &filename).await;
            }
        }
    }

    // Social login
    for p in ["google", "facebook", "github", "linkedin"] {
        set(pool, &format!("social_{p}_enabled"), if fields.contains_key(&format!("social_{p}_enabled")) { "1" } else { "0" }).await;
        if let Some(v) = fields.get(&format!("social_{p}_client_id")) {
            set(pool, &format!("social_{p}_client_id"), v).await;
        }
        if let Some(v) = fields.get(&format!("social_{p}_client_secret")) {
            if !v.is_empty() { set(pool, &format!("social_{p}_client_secret"), v).await; }
        }
    }

    // Billing — Midtrans (server_key/client_key hanya di-set bila non-empty, pola social secret)
    set(pool, "midtrans_enabled", if fields.contains_key("midtrans_enabled") { "1" } else { "0" }).await;
    set(pool, "midtrans_is_production", if fields.contains_key("midtrans_is_production") { "1" } else { "0" }).await;
    for k in ["midtrans_server_key", "midtrans_client_key"] {
        if let Some(v) = fields.get(k) {
            if !v.is_empty() { set(pool, k, v).await; }
        }
    }

    view::set_flash(&session, "success", "Settings berhasil diperbarui!").await;
    Redirect::to("/admin/settings").into_response()
}
