//! Kelola MASTER DATA Plan (paket langganan): harga, batas nomor, batas pesan/hari,
//! fitur (AI, webhook), aktif/nonaktif. Hanya Superadmin/admin (route di-gate `view_resources`).
//! Memakai shell custom `layout/wa.html`.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::types::Uuid;
use tower_sessions::Session;

use crate::{auth, rbac::CurrentUser, view, AppState};

type Row = (Uuid, String, String, i32, i32, i64, bool, bool, bool, i32);

const SELECT_ALL: &str = "SELECT id, code, name, max_sessions, max_messages_per_day, price_idr, \
     ai_enabled, webhook_enabled, is_active, sort_order FROM plans ORDER BY sort_order";

/// Format angka ribuan ala Indonesia: 99000 -> "99.000".
fn rupiah(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    let bytes = s.as_bytes();
    let len = bytes.len();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push('.');
        }
        out.push(*b as char);
    }
    if n < 0 { format!("-{out}") } else { out }
}

pub async fn index(user: CurrentUser, session: Session, State(state): State<AppState>) -> Html<String> {
    let rows: Vec<Row> = sqlx::query_as(SELECT_ALL).fetch_all(&state.pool).await.unwrap_or_default();
    // jumlah subscriber per plan
    let counts: Vec<(Uuid, i64)> = sqlx::query_as(
        "SELECT plan_id, count(*)::bigint FROM users WHERE plan_id IS NOT NULL GROUP BY plan_id",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let list: Vec<_> = rows
        .iter()
        .map(|r| {
            let subs = counts.iter().find(|c| c.0 == r.0).map(|c| c.1).unwrap_or(0);
            json!({
                "id": r.0.to_string(),
                "code": r.1,
                "name": r.2,
                "max_sessions": r.3,
                "max_messages_per_day": r.4,
                "price_idr": r.5,
                "price_fmt": rupiah(r.5),
                "ai_enabled": r.6,
                "webhook_enabled": r.7,
                "is_active": r.8,
                "subscribers": subs,
            })
        })
        .collect();
    let mut ctx = view::base_context(&state, &session, &user, "plans").await;
    ctx.insert("plans", &list);
    match state.tera.render("wa/plans.html", &ctx) {
        Ok(h) => Html(h),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

#[derive(Deserialize)]
pub struct PlanForm {
    #[serde(default)]
    name: String,
    #[serde(default)]
    max_sessions: String,
    #[serde(default)]
    max_messages_per_day: String,
    #[serde(default)]
    price_idr: String,
    #[serde(default)]
    ai_enabled: Option<String>,
    #[serde(default)]
    webhook_enabled: Option<String>,
    #[serde(default)]
    is_active: Option<String>,
    #[serde(rename = "_token", default)]
    token: String,
}

fn on(v: &Option<String>) -> bool {
    matches!(v.as_deref(), Some("on") | Some("true") | Some("1"))
}

pub async fn update(
    user: CurrentUser,
    session: Session,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(f): Form<PlanForm>,
) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    if !user.is_superadmin() && !user.can("view_resources") {
        return (StatusCode::FORBIDDEN, "forbidden").into_response();
    }
    let name = f.name.trim();
    if name.is_empty() {
        view::set_flash(&session, "error", "Nama paket tidak boleh kosong.").await;
        return Redirect::to("/admin/plans").into_response();
    }
    // Parsing angka dengan batas wajar (abaikan titik/koma agar ramah input).
    let clean = |s: &str| -> i64 { s.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0) };
    let max_sessions = clean(&f.max_sessions).clamp(0, 100000) as i32;
    let max_msgs = clean(&f.max_messages_per_day).clamp(0, 100_000_000) as i32;
    let price = clean(&f.price_idr).max(0);
    let ai = on(&f.ai_enabled);
    let wh = on(&f.webhook_enabled);
    let active = on(&f.is_active);

    let res = sqlx::query(
        "UPDATE plans SET name=$1, max_sessions=$2, max_messages_per_day=$3, price_idr=$4, \
         ai_enabled=$5, webhook_enabled=$6, is_active=$7, updated_at=now() WHERE id=$8",
    )
    .bind(name)
    .bind(max_sessions)
    .bind(max_msgs)
    .bind(price)
    .bind(ai)
    .bind(wh)
    .bind(active)
    .bind(id)
    .execute(&state.pool)
    .await;
    match res {
        Ok(_) => view::set_flash(&session, "success", &format!("Paket \"{name}\" diperbarui.")).await,
        Err(e) => view::set_flash(&session, "error", &format!("Gagal menyimpan: {e}")).await,
    }
    Redirect::to("/admin/plans").into_response()
}
