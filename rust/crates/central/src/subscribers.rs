//! Manajemen Subscriber (Superadmin): lihat semua pengguna berlangganan, paket aktif,
//! pemakaian (nomor & pesan hari ini), dan ASSIGN/ubah paket manual (tanpa harus bayar).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
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
    type Row = (Uuid, String, String, String, String, Option<NaiveDateTime>, bool, i64, i64, i32, i32);
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT u.id, u.name, u.email, COALESCE(p.name,'—'), COALESCE(p.code,''), \
                u.plan_expires_at::timestamp, \
                (u.plan_expires_at IS NULL OR u.plan_expires_at > now()) AS active, \
                (SELECT count(*) FROM wa_sessions s WHERE s.user_id=u.id) AS sessions, \
                COALESCE((SELECT sum(d.sent_count) FROM wa_sessions s JOIN wa_usage_daily d \
                          ON d.session_id=s.id AND d.day=CURRENT_DATE WHERE s.user_id=u.id),0)::bigint AS sent_today, \
                COALESCE(p.max_sessions,0), COALESCE(p.max_messages_per_day,0) \
         FROM users u LEFT JOIN plans p ON p.id=u.plan_id \
         ORDER BY u.created_at DESC NULLS LAST",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let subs: Vec<_> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.0.to_string(), "name": r.1, "email": r.2,
                "plan_name": r.3, "plan_code": r.4,
                "expires": r.5.map(|d| d.format("%d %b %Y").to_string()).unwrap_or_default(),
                "lifetime": r.5.is_none(), "active": r.6,
                "sessions": r.7, "sent_today": r.8,
                "max_sessions": r.9, "max_messages_per_day": r.10,
            })
        })
        .collect();

    let plans: Vec<(Uuid, String, String, i64)> =
        sqlx::query_as("SELECT id, code, name, price_idr FROM plans WHERE is_active ORDER BY sort_order")
            .fetch_all(&state.pool).await.unwrap_or_default();
    let plan_list: Vec<_> = plans.iter().map(|p| json!({
        "id": p.0.to_string(), "code": p.1, "name": p.2, "price": p.3
    })).collect();

    let mut ctx = view::base_context(&state, &session, &user, "subscribers").await;
    ctx.insert("subscribers", &subs);
    ctx.insert("plans", &plan_list);
    match state.tera.render("wa/subscribers.html", &ctx) {
        Ok(h) => Html(h).into_response(),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")).into_response(),
    }
}

#[derive(Deserialize)]
pub struct AssignForm {
    #[serde(default)]
    plan_id: String,
    #[serde(default)]
    days: String,
    #[serde(rename = "_token", default)]
    token: String,
}

/// Assign / ubah paket subscriber + tentukan masa berlaku (hari). days=0 → tanpa batas (paket gratis).
pub async fn set_plan(user: CurrentUser, session: Session, State(state): State<AppState>, Path(id): Path<Uuid>, Form(f): Form<AssignForm>) -> Response {
    if !user.is_superadmin() {
        return (StatusCode::FORBIDDEN, "Khusus Superadmin").into_response();
    }
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    let Ok(pid) = Uuid::parse_str(f.plan_id.trim()) else {
        view::set_flash(&session, "error", "Paket tidak valid.").await;
        return Redirect::to("/admin/wa/subscribers").into_response();
    };
    let days: i32 = f.days.trim().parse().unwrap_or(30).clamp(0, 3650);
    let res = sqlx::query(
        "UPDATE users SET plan_id=$2, plan_started_at=now(), \
            plan_expires_at = CASE WHEN $3::int > 0 THEN now() + make_interval(days => $3::int) ELSE NULL END, \
            updated_at=now() WHERE id=$1",
    )
    .bind(id).bind(pid).bind(days)
    .execute(&state.pool).await;
    match res {
        Ok(_) => view::set_flash(&session, "success", "Paket subscriber diperbarui.").await,
        Err(e) => view::set_flash(&session, "error", &format!("Gagal: {e}")).await,
    }
    Redirect::to("/admin/wa/subscribers").into_response()
}
