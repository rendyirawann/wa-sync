use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use chrono::NaiveDateTime;
use serde::Deserialize;
use std::collections::HashMap;
use sqlx::types::Uuid;
use tower_sessions::Session;

use crate::{auth, settings, AppState};

/// Context dasar untuk halaman auth (login/register/forgot/reset) + flash status/error.
async fn auth_ctx(state: &AppState, session: &Session) -> tera::Context {
    let app = settings::all(&state.pool).await;
    let mut ctx = tera::Context::new();
    ctx.insert("site_name", settings::get(&app, "site_name", "StarterTemp"));
    ctx.insert("site_logo", settings::get(&app, "site_logo", "base-logo.png"));
    ctx.insert("site_font", settings::get(&app, "site_font", "Plus Jakarta Sans"));
    ctx.insert("app_settings", &app);
    ctx.insert("current_url", "");
    ctx.insert("csrf_token", &auth::ensure_csrf(session).await);
    // flash sekali-pakai
    let status: String = session.get("auth_status").await.ok().flatten().unwrap_or_default();
    let autherror: String = session.get("auth_error").await.ok().flatten().unwrap_or_default();
    if !status.is_empty() { let _ = session.remove::<String>("auth_status").await; }
    if !autherror.is_empty() { let _ = session.remove::<String>("auth_error").await; }
    ctx.insert("status", &status);
    ctx.insert("autherror", &autherror);
    ctx
}

async fn flash(session: &Session, key: &str, msg: &str) {
    let _ = session.insert(key, msg).await;
}

fn render(state: &AppState, tpl: &str, ctx: &tera::Context) -> Response {
    match state.tera.render(tpl, ctx) {
        Ok(html) => Html(html).into_response(),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")).into_response(),
    }
}

// =================== REGISTER ===================
pub async fn register_page(State(state): State<AppState>, session: Session) -> Response {
    if matches!(session.get::<String>(auth::SESSION_USER_KEY).await, Ok(Some(_))) {
        return Redirect::to("/admin/dashboard").into_response();
    }
    let ctx = auth_ctx(&state, &session).await;
    render(&state, "auth/register.html", &ctx)
}

#[derive(Deserialize)]
pub struct RegisterForm {
    #[serde(default)] name: String,
    #[serde(default)] email: String,
    #[serde(default)] password: String,
    #[serde(default)] password_confirmation: String,
    #[serde(rename = "_token", default)] token: String,
}

pub async fn register_submit(State(state): State<AppState>, session: Session, Form(f): Form<RegisterForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        flash(&session, "auth_error", "Sesi kedaluwarsa, coba lagi.").await;
        return Redirect::to("/admin/register").into_response();
    }
    let name = f.name.trim();
    let email = f.email.trim().to_lowercase();
    let err = if name.is_empty() || email.is_empty() {
        Some("Nama dan email wajib diisi.")
    } else if !email.contains('@') {
        Some("Format email tidak valid.")
    } else if f.password.chars().count() < 8 {
        Some("Password minimal 8 karakter.")
    } else if f.password != f.password_confirmation {
        Some("Konfirmasi password tidak sama.")
    } else {
        None
    };
    if let Some(e) = err {
        flash(&session, "auth_error", e).await;
        return Redirect::to("/admin/register").into_response();
    }
    // email unik
    let exists: Option<i32> = sqlx::query_scalar("SELECT 1 FROM users WHERE email = $1").bind(&email).fetch_optional(&state.pool).await.ok().flatten();
    if exists.is_some() {
        flash(&session, "auth_error", "Email sudah terdaftar.").await;
        return Redirect::to("/admin/register").into_response();
    }
    let id = Uuid::now_v7();
    // username unik sederhana dari email
    let base_username: String = email.split('@').next().unwrap_or("user").chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').collect();
    let username = format!("{}-{}", base_username, &id.to_string()[..8]);
    let hash = bcrypt::hash(&f.password, 12).unwrap_or_default();
    let res = sqlx::query("INSERT INTO users (id, name, username, email, password, is_active, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,true,now(),now())")
        .bind(id).bind(name).bind(&username).bind(&email).bind(&hash).execute(&state.pool).await;
    if res.is_err() {
        flash(&session, "auth_error", "Gagal membuat akun.").await;
        return Redirect::to("/admin/register").into_response();
    }
    let _ = session.insert(auth::SESSION_USER_KEY, id.to_string()).await;
    Redirect::to("/admin/dashboard").into_response()
}

// =================== FORGOT PASSWORD ===================
pub async fn forgot_page(State(state): State<AppState>, session: Session) -> Response {
    let ctx = auth_ctx(&state, &session).await;
    render(&state, "auth/forgot-password.html", &ctx)
}

#[derive(Deserialize)]
pub struct ForgotForm {
    #[serde(default)] email: String,
    #[serde(rename = "_token", default)] token: String,
}

pub async fn forgot_submit(State(state): State<AppState>, session: Session, Form(f): Form<ForgotForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        flash(&session, "auth_error", "Sesi kedaluwarsa.").await;
        return Redirect::to("/admin/forgot-password").into_response();
    }
    let email = f.email.trim().to_lowercase();
    // Jika user ada → buat token + simpan + log link (dev). Selalu balas sukses (anti enumeration).
    let user: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email = $1").bind(&email).fetch_optional(&state.pool).await.ok().flatten();
    if user.is_some() {
        let token = Uuid::new_v4().to_string().replace('-', "");
        let hash = bcrypt::hash(&token, 12).unwrap_or_default();
        let _ = sqlx::query("INSERT INTO password_reset_tokens (email, token, created_at) VALUES ($1,$2,now()) ON CONFLICT (email) DO UPDATE SET token=EXCLUDED.token, created_at=now()")
            .bind(&email).bind(&hash).execute(&state.pool).await;
        let base = std::env::var("APP_URL").unwrap_or_else(|_| "http://127.0.0.1:8090".into());
        let reset_url = format!("{base}/admin/reset-password/{token}?email={email}");
        // Kirim via SMTP bila dikonfigurasi; jika tidak / gagal → fallback log link (dev).
        let sent = crate::mailer::send_reset(&email, &reset_url).await;
        if !sent {
            tracing::warn!("[PASSWORD RESET] link untuk {email}: {reset_url}");
        }
    }
    flash(&session, "auth_status", "Jika email terdaftar, tautan reset telah dikirim.").await;
    Redirect::to("/admin/forgot-password").into_response()
}

// =================== RESET PASSWORD ===================
pub async fn reset_page(State(state): State<AppState>, session: Session, Path(token): Path<String>, Query(q): Query<HashMap<String, String>>) -> Response {
    let mut ctx = auth_ctx(&state, &session).await;
    ctx.insert("token", &token);
    ctx.insert("email", q.get("email").map(String::as_str).unwrap_or(""));
    render(&state, "auth/reset-password.html", &ctx)
}

#[derive(Deserialize)]
pub struct ResetForm {
    #[serde(default)] token: String,
    #[serde(default)] email: String,
    #[serde(default)] password: String,
    #[serde(default)] password_confirmation: String,
    #[serde(rename = "_token", default)] csrf: String,
}

pub async fn reset_submit(State(state): State<AppState>, session: Session, Form(f): Form<ResetForm>) -> Response {
    if !auth::verify_csrf(&session, &f.csrf).await {
        flash(&session, "auth_error", "Sesi kedaluwarsa.").await;
        return Redirect::to(&format!("/admin/reset-password/{}?email={}", f.token, f.email)).into_response();
    }
    let email = f.email.trim().to_lowercase();
    let back = format!("/admin/reset-password/{}?email={}", f.token, email);
    if f.password.chars().count() < 8 {
        flash(&session, "auth_error", "Password minimal 8 karakter.").await;
        return Redirect::to(&back).into_response();
    }
    if f.password != f.password_confirmation {
        flash(&session, "auth_error", "Konfirmasi password tidak sama.").await;
        return Redirect::to(&back).into_response();
    }
    let row: Option<(String, Option<NaiveDateTime>)> = sqlx::query_as("SELECT token, created_at FROM password_reset_tokens WHERE email = $1").bind(&email).fetch_optional(&state.pool).await.ok().flatten();
    let Some((stored, created)) = row else {
        flash(&session, "auth_error", "Token reset tidak valid.").await;
        return Redirect::to(&back).into_response();
    };
    let expired = created.map(|c| (chrono::Local::now().naive_local() - c).num_minutes() > 60).unwrap_or(true);
    if expired || !bcrypt::verify(&f.token, &stored).unwrap_or(false) {
        flash(&session, "auth_error", "Token reset tidak valid atau kedaluwarsa.").await;
        return Redirect::to(&back).into_response();
    }
    let hash = bcrypt::hash(&f.password, 12).unwrap_or_default();
    let _ = sqlx::query("UPDATE users SET password = $1, updated_at = now() WHERE email = $2").bind(&hash).bind(&email).execute(&state.pool).await;
    let _ = sqlx::query("DELETE FROM password_reset_tokens WHERE email = $1").bind(&email).execute(&state.pool).await;
    flash(&session, "auth_status", "Password berhasil direset. Silakan login.").await;
    Redirect::to("/admin/login").into_response()
}

// =================== PASSWORD UPDATE (scaffolding, minimal) ===================
#[derive(Deserialize)]
pub struct PwUpdateForm {
    #[serde(default)] current_password: String,
    #[serde(default)] password: String,
    #[serde(default)] password_confirmation: String,
    #[serde(rename = "_token", default)] token: String,
}

pub async fn password_update(session: Session, State(state): State<AppState>, Form(f): Form<PwUpdateForm>) -> Response {
    let uid: Option<String> = session.get(auth::SESSION_USER_KEY).await.ok().flatten();
    let Some(uid) = uid.and_then(|s| Uuid::parse_str(&s).ok()) else {
        return Redirect::to("/admin/login").into_response();
    };
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), Html("<h1>419</h1>")).into_response();
    }
    let cur: String = sqlx::query_scalar("SELECT password FROM users WHERE id=$1").bind(uid).fetch_one(&state.pool).await.unwrap_or_default();
    if !bcrypt::verify(&f.current_password, &cur).unwrap_or(false) {
        flash(&session, "flash_error", "Password saat ini salah.").await;
        return Redirect::to("/admin/my-security").into_response();
    }
    if f.password.chars().count() < 8 || f.password != f.password_confirmation {
        flash(&session, "flash_error", "Password baru tidak valid.").await;
        return Redirect::to("/admin/my-security").into_response();
    }
    let hash = bcrypt::hash(&f.password, 12).unwrap_or_default();
    let _ = sqlx::query("UPDATE users SET password=$1 WHERE id=$2").bind(&hash).bind(uid).execute(&state.pool).await;
    flash(&session, "flash_success", "Password berhasil diperbarui.").await;
    Redirect::to("/admin/my-security").into_response()
}

// =================== EMAIL VERIFY / CONFIRM PASSWORD (stub: tak dipakai aktif) ===================
pub async fn verify_notice() -> Response { Redirect::to("/admin/dashboard").into_response() }
pub async fn verify_resend() -> Response { Redirect::to("/admin/dashboard").into_response() }
pub async fn verify_link(session: Session, State(state): State<AppState>, Path((id, _hash)): Path<(String, String)>) -> Response {
    if let Ok(uid) = Uuid::parse_str(&id) {
        let _ = sqlx::query("UPDATE users SET email_verified_at = now() WHERE id=$1 AND email_verified_at IS NULL").bind(uid).execute(&state.pool).await;
    }
    let _ = &session;
    Redirect::to("/admin/dashboard?verified=1").into_response()
}
pub async fn confirm_show() -> Response { Redirect::to("/admin/dashboard").into_response() }
pub async fn confirm_store() -> Response { Redirect::to("/admin/dashboard").into_response() }
