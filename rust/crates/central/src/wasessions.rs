//! Halaman & aksi "Sesi WhatsApp" (Fase B P2). Semua di-scope per-user
//! (Superadmin lihat/kelola semua). Memanggil sidecar lewat `state.wa`.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Form, Json,
};
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_json::json;
use sqlx::types::Uuid;
use tower_sessions::Session;

use crate::{auth, rbac::CurrentUser, view, AppState};

type Row = (Uuid, String, Option<String>, String, Option<NaiveDateTime>, String, bool, String, String, bool);

const SELECT_ALL: &str =
    "SELECT id, label, phone, status, last_connected_at::timestamp, antiban_level, ai_enabled, ai_model, ai_system_prompt, sim_mode FROM wa_sessions ORDER BY created_at DESC";
const SELECT_OWN: &str =
    "SELECT id, label, phone, status, last_connected_at::timestamp, antiban_level, ai_enabled, ai_model, ai_system_prompt, sim_mode FROM wa_sessions WHERE user_id=$1 ORDER BY created_at DESC";

pub async fn index(user: CurrentUser, session: Session, State(state): State<AppState>) -> Html<String> {
    let rows: Vec<Row> = if user.is_superadmin() {
        sqlx::query_as(SELECT_ALL).fetch_all(&state.pool).await.unwrap_or_default()
    } else {
        sqlx::query_as(SELECT_OWN).bind(user.id).fetch_all(&state.pool).await.unwrap_or_default()
    };
    let list: Vec<_> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.0.to_string(),
                "label": r.1,
                "phone": r.2,
                "status": r.3,
                "last": r.4.map(crate::activity::human_diff).unwrap_or_else(|| "—".into()),
                "level": r.5,
                "ai_enabled": r.6,
                "ai_model": r.7,
                "ai_prompt": r.8,
                "sim": r.9,
            })
        })
        .collect();
    let used = rows.len() as i64;
    let max_sessions = if user.is_superadmin() { 0 } else { crate::quota::plan_limits(&state.pool, user.id).await.0 };
    let mut ctx = view::base_context(&state, &session, &user, "sessions").await;
    ctx.insert("wa_sessions", &list);
    ctx.insert("sessions_used", &used);
    ctx.insert("sessions_max", &max_sessions);
    match state.tera.render("wa/sessions.html", &ctx) {
        Ok(h) => Html(h),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

/// True bila user pemilik sesi (atau Superadmin) dan sesi ada.
async fn owns(state: &AppState, user: &CurrentUser, id: Uuid) -> bool {
    let found: Option<(Uuid,)> = if user.is_superadmin() {
        sqlx::query_as("SELECT id FROM wa_sessions WHERE id=$1").bind(id)
            .fetch_optional(&state.pool).await.ok().flatten()
    } else {
        sqlx::query_as("SELECT id FROM wa_sessions WHERE id=$1 AND user_id=$2").bind(id).bind(user.id)
            .fetch_optional(&state.pool).await.ok().flatten()
    };
    found.is_some()
}

#[derive(Deserialize)]
pub struct CreateForm {
    #[serde(default)]
    label: String,
    #[serde(default)]
    level: String,
    #[serde(default)]
    sim: Option<String>,
    #[serde(rename = "_token", default)]
    token: String,
}

pub async fn create(user: CurrentUser, session: Session, State(state): State<AppState>, Form(f): Form<CreateForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    // Enforcement kuota nomor per-plan (Superadmin bypass).
    if !user.is_superadmin() {
        let (max_sessions, _, _, _) = crate::quota::plan_limits(&state.pool, user.id).await;
        if crate::quota::session_count(&state.pool, user.id).await >= max_sessions {
            view::set_flash(&session, "error", "Kuota nomor untuk paket Anda sudah penuh. Upgrade plan untuk menambah nomor.").await;
            return Redirect::to("/admin/wa/sessions").into_response();
        }
    }
    let id = Uuid::now_v7();
    let label = if f.label.trim().is_empty() {
        format!("Sesi {}", &id.to_string()[..8])
    } else {
        f.label.trim().to_string()
    };
    // Level anti-ban dipilih saat tautkan (default normal bila tak valid).
    let level = match f.level.as_str() {
        "safe" | "normal" | "aggressive" => f.level.as_str(),
        _ => "normal",
    };
    let sim = matches!(f.sim.as_deref(), Some("on") | Some("true") | Some("1"));
    // Kredensial per-nomor (auth_key) + secret webhook — dibuat sekali saat tautkan.
    let auth_key = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let webhook_secret = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let _ = sqlx::query("INSERT INTO wa_sessions (id, user_id, label, status, antiban_level, sim_mode, auth_key, webhook_secret) VALUES ($1,$2,$3,'pending',$4,$5,$6,$7)")
        .bind(id).bind(user.id).bind(&label).bind(level).bind(sim).bind(&auth_key).bind(&webhook_secret).execute(&state.pool).await;
    // mulai sesi di sidecar — QR (real) / langsung connected (sim) datang via /internal/wa/event
    if let Err(e) = state.wa.start(&id.to_string(), &user.id.to_string(), level, sim).await {
        tracing::warn!("gagal start sesi WA {id}: {e}");
        view::set_flash(&session, "warning", "Nomor dibuat, tapi gateway WhatsApp (sidecar :8099) belum aktif. Jalankan .\\dev-all.ps1, lalu buka QR pada nomor ini.").await;
    } else if sim {
        view::set_flash(&session, "success", "Nomor SIMULASI dibuat & otomatis tersambung (tanpa scan QR). Gunakan tombol 'Simulasi pesan masuk' untuk menguji.").await;
    }
    Redirect::to("/admin/wa/sessions").into_response()
}

pub async fn qr(user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    if !owns(&state, &user, id).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    match state.wa.qr(&id.to_string()).await {
        Ok(q) => Json(json!({ "status": q.status, "qr": q.qr })).into_response(),
        Err(_) => Json(json!({ "status": "unknown", "qr": null })).into_response(),
    }
}

pub async fn restart(user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    if !owns(&state, &user, id).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let row: (String, bool) = sqlx::query_as("SELECT antiban_level, sim_mode FROM wa_sessions WHERE id=$1")
        .bind(id).fetch_one(&state.pool).await.unwrap_or_else(|_| ("normal".into(), false));
    let _ = state.wa.start(&id.to_string(), &user.id.to_string(), &row.0, row.1).await;
    Json(json!({ "status": "ok" })).into_response()
}

#[derive(Deserialize)]
pub struct SimForm {
    #[serde(default)]
    from: String,
    #[serde(default)]
    text: String,
    #[serde(rename = "_token", default)]
    token: String,
}

/// Suntik pesan masuk SIMULASI ke sebuah nomor (mode sim) — uji E2E tanpa nomor WA nyata.
pub async fn simulate(user: CurrentUser, session: Session, State(state): State<AppState>, Path(id): Path<Uuid>, Form(f): Form<SimForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    if !owns(&state, &user, id).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let from = if f.from.trim().is_empty() { "628000000001".to_string() } else { f.from.trim().to_string() };
    let text = if f.text.trim().is_empty() { "Halo, ini pesan uji simulasi.".to_string() } else { f.text.trim().to_string() };
    match state.wa.sim_incoming(&id.to_string(), &from, &text).await {
        Ok(_) => view::set_flash(&session, "success", "Pesan masuk simulasi terkirim. Cek Inbox / balasan AI.").await,
        Err(_) => view::set_flash(&session, "error", "Gagal: nomor ini bukan mode simulasi, atau sidecar tidak aktif.").await,
    }
    Redirect::to("/admin/wa/sessions").into_response()
}

#[derive(Deserialize)]
pub struct LevelForm {
    #[serde(default)]
    level: String,
    #[serde(rename = "_token", default)]
    token: String,
}

/// Ubah level anti-ban nomor (safe/normal/aggressive) — simpan DB + push live ke sidecar.
pub async fn set_level(user: CurrentUser, session: Session, State(state): State<AppState>, Path(id): Path<Uuid>, Form(f): Form<LevelForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    if !owns(&state, &user, id).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let level = match f.level.as_str() {
        "safe" | "normal" | "aggressive" => f.level.as_str(),
        _ => "normal",
    };
    let _ = sqlx::query("UPDATE wa_sessions SET antiban_level=$1, updated_at=now() WHERE id=$2")
        .bind(level).bind(id).execute(&state.pool).await;
    let _ = state.wa.set_level(&id.to_string(), level).await;
    view::set_flash(&session, "success", &format!("Level anti-ban nomor diubah ke '{level}'.")).await;
    Redirect::to("/admin/wa/sessions").into_response()
}

#[derive(Deserialize)]
pub struct WebhookForm {
    #[serde(default)]
    webhook_url: String,
    #[serde(default)]
    enabled: Option<String>,
    #[serde(rename = "_token", default)]
    token: String,
}

/// Simpan konfigurasi webhook keluar untuk satu nomor.
pub async fn set_webhook(user: CurrentUser, session: Session, State(state): State<AppState>, Path(id): Path<Uuid>, Form(f): Form<WebhookForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    if !owns(&state, &user, id).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let enabled = matches!(f.enabled.as_deref(), Some("on") | Some("true") | Some("1"));
    let url = f.webhook_url.trim();
    let url_opt: Option<&str> = if url.is_empty() { None } else { Some(url) };
    let _ = sqlx::query("UPDATE wa_sessions SET webhook_url=$1, webhook_enabled=$2, updated_at=now() WHERE id=$3")
        .bind(url_opt).bind(enabled).bind(id).execute(&state.pool).await;
    view::set_flash(&session, "success", "Konfigurasi webhook disimpan.").await;
    Redirect::to("/admin/wa/webhooks").into_response()
}

#[derive(Deserialize)]
pub struct AiForm {
    #[serde(default)]
    ai_enabled: Option<String>,
    #[serde(default)]
    ai_model: String,
    #[serde(default)]
    ai_system_prompt: String,
    #[serde(rename = "_token", default)]
    token: String,
}

/// Simpan konfigurasi AI chatbot untuk satu nomor.
pub async fn set_ai(user: CurrentUser, session: Session, State(state): State<AppState>, Path(id): Path<Uuid>, Form(f): Form<AiForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    if !owns(&state, &user, id).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let enabled = matches!(f.ai_enabled.as_deref(), Some("on") | Some("true") | Some("1"));
    let model = match f.ai_model.trim() {
        "" => "qwen2.5".to_string(),
        m => m.to_string(),
    };
    let _ = sqlx::query("UPDATE wa_sessions SET ai_enabled=$1, ai_model=$2, ai_system_prompt=$3, updated_at=now() WHERE id=$4")
        .bind(enabled).bind(&model).bind(f.ai_system_prompt.trim()).bind(id).execute(&state.pool).await;
    view::set_flash(&session, "success", if enabled { "AI chatbot diaktifkan untuk nomor ini." } else { "AI chatbot dimatikan." }).await;
    Redirect::to("/admin/wa/sessions").into_response()
}

pub async fn logout(user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    if !owns(&state, &user, id).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let _ = state.wa.logout(&id.to_string()).await;
    let _ = sqlx::query("UPDATE wa_sessions SET status='logged_out', qr=NULL, updated_at=now() WHERE id=$1")
        .bind(id).execute(&state.pool).await;
    Json(json!({ "status": "ok" })).into_response()
}

pub async fn destroy(user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    if !owns(&state, &user, id).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let _ = state.wa.delete(&id.to_string(), true).await;
    let _ = sqlx::query("DELETE FROM wa_sessions WHERE id=$1").bind(id).execute(&state.pool).await;
    Json(json!({ "status": "ok" })).into_response()
}
