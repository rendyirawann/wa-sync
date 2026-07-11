//! Kelola kupon diskon (Superadmin). Dipakai di checkout billing (persen off).

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

pub async fn index(user: CurrentUser, session: Session, State(state): State<AppState>) -> Response {
    if !user.is_superadmin() {
        return (StatusCode::FORBIDDEN, "Khusus Superadmin").into_response();
    }
    type Row = (Uuid, String, i32, bool, Option<NaiveDateTime>, i32, i32);
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, code, percent, active, expires_at::timestamp, max_uses, used_count FROM wa_coupons ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool).await.unwrap_or_default();
    let coupons: Vec<_> = rows.iter().map(|r| json!({
        "id": r.0.to_string(), "code": r.1, "percent": r.2, "active": r.3,
        "expires": r.4.map(|d| d.format("%d %b %Y").to_string()).unwrap_or_default(),
        "max_uses": r.5, "used": r.6
    })).collect();
    let mut ctx = view::base_context(&state, &session, &user, "coupons").await;
    ctx.insert("coupons", &coupons);
    match state.tera.render("coupons.html", &ctx) {
        Ok(h) => Html(h).into_response(),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")).into_response(),
    }
}

#[derive(Deserialize)]
pub struct CouponForm {
    #[serde(default)]
    code: String,
    #[serde(default)]
    percent: String,
    #[serde(default)]
    max_uses: String,
    #[serde(default)]
    expires_at: String,
    #[serde(rename = "_token", default)]
    token: String,
}

pub async fn create(user: CurrentUser, session: Session, State(state): State<AppState>, Form(f): Form<CouponForm>) -> Response {
    if !user.is_superadmin() {
        return (StatusCode::FORBIDDEN, "Khusus Superadmin").into_response();
    }
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    let code = f.code.trim().to_uppercase();
    let percent: i32 = f.percent.trim().parse().unwrap_or(0).clamp(1, 100);
    let max_uses: i32 = f.max_uses.trim().parse().unwrap_or(0).max(0);
    if code.is_empty() {
        view::set_flash(&session, "error", "Kode kupon wajib diisi.").await;
        return Redirect::to("/admin/coupons").into_response();
    }
    // expires_at (datetime-local) → biarkan Postgres cast; kosong = NULL.
    let exp = f.expires_at.trim();
    let res = if exp.is_empty() {
        sqlx::query("INSERT INTO wa_coupons (code, percent, max_uses) VALUES ($1,$2,$3) ON CONFLICT (code) DO UPDATE SET percent=EXCLUDED.percent, max_uses=EXCLUDED.max_uses, active=true")
            .bind(&code).bind(percent).bind(max_uses).execute(&state.pool).await
    } else {
        sqlx::query("INSERT INTO wa_coupons (code, percent, max_uses, expires_at) VALUES ($1,$2,$3,$4::timestamptz) ON CONFLICT (code) DO UPDATE SET percent=EXCLUDED.percent, max_uses=EXCLUDED.max_uses, expires_at=EXCLUDED.expires_at, active=true")
            .bind(&code).bind(percent).bind(max_uses).bind(exp.replace('T', " ")).execute(&state.pool).await
    };
    match res {
        Ok(_) => view::set_flash(&session, "success", &format!("Kupon {code} disimpan.")).await,
        Err(e) => view::set_flash(&session, "error", &format!("Gagal: {e}")).await,
    }
    Redirect::to("/admin/coupons").into_response()
}

pub async fn destroy(user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    if !user.is_superadmin() {
        return (StatusCode::FORBIDDEN, "forbidden").into_response();
    }
    let _ = sqlx::query("DELETE FROM wa_coupons WHERE id=$1").bind(id).execute(&state.pool).await;
    Json(json!({ "status": "ok" })).into_response()
}
