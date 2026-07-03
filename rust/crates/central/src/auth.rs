use std::collections::HashMap;
use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Form, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::Uuid;
use tower_sessions::Session;

use crate::{settings, AppState};

pub const SESSION_USER_KEY: &str = "user_id";
pub const SESSION_CSRF_KEY: &str = "csrf_token";

/// Ambil token CSRF dari session; buat baru bila belum ada.
pub async fn ensure_csrf(session: &Session) -> String {
    if let Ok(Some(tok)) = session.get::<String>(SESSION_CSRF_KEY).await {
        return tok;
    }
    let tok = Uuid::new_v4().to_string();
    let _ = session.insert(SESSION_CSRF_KEY, &tok).await;
    tok
}

/// Verifikasi token CSRF dari form terhadap yang tersimpan di session.
pub async fn verify_csrf(session: &Session, token: &str) -> bool {
    !token.is_empty()
        && matches!(session.get::<String>(SESSION_CSRF_KEY).await, Ok(Some(t)) if t == token)
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
    #[serde(rename = "_token", default)]
    pub csrf: String,
    #[serde(default)]
    pub remember: Option<String>,
}

#[derive(sqlx::FromRow)]
struct AuthUser {
    id: Uuid,
    #[allow(dead_code)]
    name: String,
    password: String,
    banned_at: Option<sqlx::types::chrono::NaiveDateTime>,
    is_active: bool,
}

/// Metadata penyedia social login (untuk tombol di halaman login).
#[derive(Serialize)]
pub struct SocialProvider {
    pub label: String,
    pub driver: String,
    pub icon: String,
    pub border: String,
}

const ICON_GOOGLE: &str = r##"<svg width="20" height="20" viewBox="0 0 24 24"><path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 01-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" fill="#4285F4"/><path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/><path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/><path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/></svg>"##;
const ICON_FACEBOOK: &str = r##"<svg width="20" height="20" viewBox="0 0 24 24"><path d="M24 12.073c0-6.627-5.373-12-12-12s-12 5.373-12 12c0 5.99 4.388 10.954 10.125 11.854v-8.385H7.078v-3.47h3.047V9.43c0-3.007 1.792-4.669 4.533-4.669 1.312 0 2.686.235 2.686.235v2.953H15.83c-1.491 0-1.956.925-1.956 1.874v2.25h3.328l-.532 3.47h-2.796v8.385C19.612 23.027 24 18.062 24 12.073z" fill="#1877F2"/></svg>"##;
const ICON_GITHUB: &str = r##"<svg width="20" height="20" viewBox="0 0 24 24"><path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" fill="#333"/></svg>"##;
const ICON_LINKEDIN: &str = r##"<svg width="20" height="20" viewBox="0 0 24 24"><path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z" fill="#0A66C2"/></svg>"##;

/// Bangun daftar penyedia social yang AKTIF (berdasarkan setting `social_{x}_enabled`).
fn enabled_social_providers(s: &HashMap<String, String>) -> Vec<SocialProvider> {
    let defs = [
        ("google", "Google", "google", ICON_GOOGLE, "#dadce0"),
        ("facebook", "Facebook", "facebook", ICON_FACEBOOK, "#1877F2"),
        ("github", "GitHub", "github", ICON_GITHUB, "#24292f"),
        ("linkedin", "LinkedIn", "linkedin-openid", ICON_LINKEDIN, "#0A66C2"),
    ];
    defs.iter()
        .filter(|(key, ..)| s.get(&format!("social_{key}_enabled")).map(String::as_str) == Some("1"))
        .map(|(_, label, driver, icon, border)| SocialProvider {
            label: (*label).into(),
            driver: (*driver).into(),
            icon: (*icon).into(),
            border: (*border).into(),
        })
        .collect()
}

fn render(state: &AppState, tpl: &str, ctx: &tera::Context) -> Response {
    match state.tera.render(tpl, ctx) {
        Ok(html) => Html(html).into_response(),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")).into_response(),
    }
}

/// GET /admin/login — tampilkan halaman login (atau redirect bila sudah login).
pub async fn login_page(State(state): State<AppState>, session: Session) -> Response {
    if is_logged_in(&session).await {
        return Redirect::to("/admin/dashboard").into_response();
    }
    let app_settings = settings::all(&state.pool).await;
    let providers = enabled_social_providers(&app_settings);

    let mut ctx = tera::Context::new();
    ctx.insert("site_name", settings::get(&app_settings, "site_name", "StarterTemp"));
    ctx.insert("site_logo", settings::get(&app_settings, "site_logo", "base-logo.png"));
    ctx.insert("site_font", settings::get(&app_settings, "site_font", "Plus Jakarta Sans"));
    ctx.insert("app_settings", &app_settings);
    ctx.insert("current_url", "/admin/login");
    ctx.insert("csrf_token", &ensure_csrf(&session).await);
    ctx.insert("has_any_social", &!providers.is_empty());
    ctx.insert("social_providers", &providers);
    render(&state, "auth/login.html", &ctx)
}

/// POST /admin/login — autentikasi multi-field + bcrypt, dengan CSRF & rate-limit.
pub async fn login_submit(
    State(state): State<AppState>,
    session: Session,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Form(form): Form<LoginForm>,
) -> Response {
    // 1. CSRF
    if !verify_csrf(&session, &form.csrf).await {
        return error_json(
            StatusCode::from_u16(419).unwrap_or(StatusCode::FORBIDDEN),
            "Sesi kedaluwarsa, silakan muat ulang halaman.",
        );
    }

    let login = form.email.trim();
    let ip = addr.ip().to_string();
    let key = format!("{}|{}", login.to_lowercase(), ip);

    // 2. Sedang dalam masa lockout?
    if let Some(secs) = state.limiter.locked_seconds(&key) {
        return lockout_json(secs);
    }

    // 3. Deteksi field login (email / no_wa / name) seperti LoginRequest Laravel.
    let user = if login.contains('@') {
        find_user(&state, "SELECT id, name, password, banned_at, is_active FROM users WHERE email = $1", login).await
    } else if !login.is_empty() && login.chars().all(|c| c.is_ascii_digit()) {
        find_user(&state, "SELECT id, name, password, banned_at, is_active FROM users WHERE no_wa = $1", login).await
    } else {
        find_user(&state, "SELECT id, name, password, banned_at, is_active FROM users WHERE name = $1", login).await
    };

    let Some(user) = user else {
        return invalid_credentials(&state, &key);
    };
    if !bcrypt::verify(&form.password, &user.password).unwrap_or(false) {
        return invalid_credentials(&state, &key);
    }

    // 4. Password benar → reset limiter.
    state.limiter.clear(&key);

    if user.banned_at.is_some() {
        return error_json(StatusCode::FORBIDDEN, "Akun Anda telah dibekukan.");
    }
    if !user.is_active {
        return error_json(StatusCode::FORBIDDEN, "Akun Anda tidak aktif.");
    }

    // 5. Update last login + catat activity log.
    let ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let _ = sqlx::query("UPDATE users SET last_ip = $1, last_login = now() WHERE id = $2")
        .bind(&ip)
        .bind(user.id)
        .execute(&state.pool)
        .await;
    log_activity(&state, "login", "Login berhasil", user.id, &ip, &ua).await;

    // 6. Simpan session.
    if session.insert(SESSION_USER_KEY, user.id.to_string()).await.is_err() {
        return error_json(StatusCode::INTERNAL_SERVER_ERROR, "Gagal membuat sesi.");
    }

    Json(json!({
        "status": "success",
        "message": "Login berhasil, mengalihkan...",
        "redirect": "/admin/dashboard"
    }))
    .into_response()
}

/// GET/POST /admin/logout — catat log lalu hapus session.
pub async fn logout(
    State(state): State<AppState>,
    session: Session,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Response {
    if let Ok(Some(uid)) = session.get::<String>(SESSION_USER_KEY).await {
        if let Ok(user_id) = Uuid::parse_str(&uid) {
            let ip = addr.ip().to_string();
            let ua = headers
                .get("user-agent")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            log_activity(&state, "logout", "Logout berhasil", user_id, &ip, &ua).await;
        }
    }
    let _ = session.flush().await;
    Redirect::to("/admin/login").into_response()
}

/// Middleware: tolak akses bila belum login.
pub async fn require_auth(session: Session, req: Request, next: Next) -> Response {
    if is_logged_in(&session).await {
        next.run(req).await
    } else {
        Redirect::to("/admin/login").into_response()
    }
}

async fn is_logged_in(session: &Session) -> bool {
    matches!(session.get::<String>(SESSION_USER_KEY).await, Ok(Some(_)))
}

async fn find_user(state: &AppState, sql: &'static str, value: &str) -> Option<AuthUser> {
    sqlx::query_as::<_, AuthUser>(sql)
        .bind(value)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten()
}

/// Catat ke tabel activity_log (spatie/activitylog) — versi ringkas.
async fn log_activity(
    state: &AppState,
    log_name: &str,
    description: &str,
    user_id: Uuid,
    ip: &str,
    user_agent: &str,
) {
    let props = json!({ "ip": ip, "agent": { "raw": user_agent } }).to_string();
    let _ = sqlx::query(
        "INSERT INTO activity_log (log_name, event, description, causer_type, causer_id, properties, created_at, updated_at) \
         VALUES ($1, NULL, $2, $3, $4, $5::json, now(), now())",
    )
    .bind(log_name)
    .bind(description)
    .bind("App\\Models\\User")
    .bind(user_id)
    .bind(props)
    .execute(&state.pool)
    .await;
}

fn invalid_credentials(state: &AppState, key: &str) -> Response {
    match state.limiter.record_failure(key) {
        Some(secs) => lockout_json(secs),
        None => error_json(StatusCode::UNPROCESSABLE_ENTITY, "Akun atau Password salah."),
    }
}

fn lockout_json(secs: u64) -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(json!({
            "errors": {
                "email": [format!("Terlalu banyak percobaan. Tunggu {secs} detik.")],
                "seconds": [secs]
            }
        })),
    )
        .into_response()
}

fn error_json(status: StatusCode, msg: &str) -> Response {
    (status, Json(json!({ "errors": { "email": [msg] } }))).into_response()
}
