//! Halaman modul WA Service: API Keys & Webhooks (shell custom `layout/wa.html`).

use axum::{extract::State, response::Html};
use serde_json::json;
use sqlx::types::Uuid;
use tower_sessions::Session;

use crate::{rbac::CurrentUser, view, AppState};

pub async fn api_keys(user: CurrentUser, session: Session, State(state): State<AppState>) -> Html<String> {
    // app_key = per akun (sama untuk semua nomor user); auth_key = per nomor.
    let app_key: String = sqlx::query_scalar("SELECT app_key::text FROM users WHERE id=$1")
        .bind(user.id).fetch_one(&state.pool).await.unwrap_or_default();
    let rows: Vec<(Uuid, String, Option<String>, Option<String>, String)> = if user.is_superadmin() {
        sqlx::query_as("SELECT id, label, phone, auth_key, status FROM wa_sessions ORDER BY created_at DESC")
            .fetch_all(&state.pool).await.unwrap_or_default()
    } else {
        sqlx::query_as("SELECT id, label, phone, auth_key, status FROM wa_sessions WHERE user_id=$1 ORDER BY created_at DESC")
            .bind(user.id).fetch_all(&state.pool).await.unwrap_or_default()
    };
    let nums: Vec<_> = rows.iter().map(|r| json!({
        "id": r.0.to_string(), "label": r.1, "phone": r.2,
        "auth_key": r.3.clone().unwrap_or_default(), "status": r.4
    })).collect();
    let base = std::env::var("APP_URL").unwrap_or_else(|_| "http://127.0.0.1:8090".into());
    let mut ctx = view::base_context(&state, &session, &user, "api-keys").await;
    ctx.insert("app_key", &app_key);
    ctx.insert("api_numbers", &nums);
    ctx.insert("api_base", &base);
    match state.tera.render("wa/api_keys.html", &ctx) {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

pub async fn webhooks(user: CurrentUser, session: Session, State(state): State<AppState>) -> Html<String> {
    let rows: Vec<(Uuid, String, Option<String>, Option<String>, bool, Option<String>)> = if user.is_superadmin() {
        sqlx::query_as("SELECT id, label, phone, webhook_url, webhook_enabled, webhook_secret FROM wa_sessions ORDER BY created_at DESC")
            .fetch_all(&state.pool).await.unwrap_or_default()
    } else {
        sqlx::query_as("SELECT id, label, phone, webhook_url, webhook_enabled, webhook_secret FROM wa_sessions WHERE user_id=$1 ORDER BY created_at DESC")
            .bind(user.id).fetch_all(&state.pool).await.unwrap_or_default()
    };
    let nums: Vec<_> = rows.iter().map(|r| json!({
        "id": r.0.to_string(), "label": r.1, "phone": r.2,
        "url": r.3.clone().unwrap_or_default(), "enabled": r.4, "secret": r.5.clone().unwrap_or_default()
    })).collect();
    let mut ctx = view::base_context(&state, &session, &user, "webhooks").await;
    ctx.insert("wh_numbers", &nums);
    match state.tera.render("wa/webhooks.html", &ctx) {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

