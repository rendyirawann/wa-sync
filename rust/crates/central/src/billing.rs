//! Billing & Plan (Fase P5): subscriber lihat paket aktif, upgrade/perpanjang via Midtrans
//! Snap, dan aktivasi plan otomatis setelah pembayaran lunas (notifikasi server-to-server).
//!
//! Keamanan: signature SHA512 (midtrans::verify_signature), idempoten (order_id UNIQUE +
//! guard applied_at), anti-tamper harga (harga dari `wa_invoices`, bukan dari payload).

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Form, Json,
};
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::types::Uuid;
use tower_sessions::Session;

use crate::{auth, midtrans, rbac::CurrentUser, settings, view, AppState};

/// Job berkala: turunkan user yang masa berlakunya habis kembali ke paket Basic (gratis).
/// Dipanggil saat boot + tiap jam dari main.rs. `quota::plan_limits` juga sudah menjaring
/// plan kadaluarsa secara langsung; job ini merapikan `plan_id` agar tampilan konsisten.
pub async fn downgrade_expired(pool: &sqlx::PgPool) {
    let res = sqlx::query(
        "UPDATE users SET plan_id = (SELECT id FROM plans WHERE code='basic'), \
            plan_started_at = NULL, plan_expires_at = NULL, updated_at = now() \
         WHERE plan_expires_at IS NOT NULL AND plan_expires_at < now() \
           AND plan_id IS DISTINCT FROM (SELECT id FROM plans WHERE code='basic')",
    )
    .execute(pool)
    .await;
    if let Ok(r) = res {
        if r.rows_affected() > 0 {
            tracing::info!("Auto-downgrade {} user kadaluarsa → Basic", r.rows_affected());
        }
    }
}

/// Format ribuan ala Indonesia: 99000 -> "99.000".
fn rupiah(n: i64) -> String {
    let s = n.abs().to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut out = String::new();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push('.');
        }
        out.push(*b as char);
    }
    out
}

#[derive(Deserialize)]
pub struct IndexQ {
    #[serde(default)]
    status: String,
}

pub async fn index(user: CurrentUser, session: Session, State(state): State<AppState>, Query(q): Query<IndexQ>) -> Html<String> {
    // Paket aktif user + status masa berlaku (dihitung di SQL).
    type Cur = (String, String, i32, i32, bool, bool, Option<NaiveDateTime>, bool, i32);
    let cur: Option<Cur> = sqlx::query_as(
        "SELECT COALESCE(p.code,'basic'), COALESCE(p.name,'-'), COALESCE(p.max_sessions,0), \
                COALESCE(p.max_messages_per_day,0), COALESCE(p.ai_enabled,false), COALESCE(p.webhook_enabled,false), \
                u.plan_expires_at::timestamp, \
                (u.plan_expires_at IS NULL OR u.plan_expires_at > now()) AS active, \
                GREATEST(0, CEIL(EXTRACT(EPOCH FROM (COALESCE(u.plan_expires_at, now()) - now()))/86400.0))::int AS days_left \
         FROM users u LEFT JOIN plans p ON p.id=u.plan_id WHERE u.id=$1",
    )
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    let current = cur.map(|c| {
        json!({
            "code": c.0, "name": c.1, "max_sessions": c.2, "max_messages_per_day": c.3,
            "ai_enabled": c.4, "webhook_enabled": c.5,
            "expires": c.6.map(|d| d.format("%d %b %Y").to_string()).unwrap_or_default(),
            "lifetime": c.6.is_none(), "active": c.7, "days_left": c.8,
        })
    });
    let current_code = current.as_ref().and_then(|c| c.get("code")).and_then(|v| v.as_str()).unwrap_or("").to_string();

    // Konfigurasi Midtrans (untuk enable/disable tombol bayar).
    let s = settings::all(&state.pool).await;
    let configured = midtrans::MidtransCfg::from_settings(&s).is_configured();

    // Paket tersedia.
    type P = (Uuid, String, String, i32, i32, i64, bool, bool);
    let plans: Vec<P> = sqlx::query_as(
        "SELECT id, code, name, max_sessions, max_messages_per_day, price_idr, ai_enabled, webhook_enabled \
         FROM plans WHERE is_active ORDER BY sort_order",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let plan_list: Vec<_> = plans
        .iter()
        .map(|p| {
            json!({
                "id": p.0.to_string(), "code": p.1, "name": p.2,
                "max_sessions": p.3, "max_messages_per_day": p.4,
                "price": p.5, "price_fmt": rupiah(p.5),
                "ai_enabled": p.6, "webhook_enabled": p.7,
                "is_current": p.1 == current_code,
                "buyable": p.5 > 0 && configured && !user.is_superadmin(),
            })
        })
        .collect();

    // Riwayat invoice.
    type Inv = (String, String, i64, String, NaiveDateTime);
    let invs: Vec<Inv> = sqlx::query_as(
        "SELECT order_id, plan_name, amount_idr, status, created_at::timestamp FROM wa_invoices \
         WHERE user_id=$1 ORDER BY created_at DESC LIMIT 50",
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let invoices: Vec<_> = invs
        .iter()
        .map(|i| json!({
            "order_id": i.0, "plan": i.1, "amount_fmt": rupiah(i.2),
            "status": i.3, "created": i.4.format("%d %b %Y, %H:%M").to_string(),
        }))
        .collect();

    let mut ctx = view::base_context(&state, &session, &user, "billing").await;
    ctx.insert("current", &current);
    ctx.insert("plans", &plan_list);
    ctx.insert("invoices", &invoices);
    ctx.insert("midtrans_configured", &configured);
    ctx.insert("is_super", &user.is_superadmin());
    ctx.insert("pay_status", &q.status);
    match state.tera.render("wa/billing.html", &ctx) {
        Ok(h) => Html(h),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

#[derive(Deserialize)]
pub struct CheckoutForm {
    #[serde(default)]
    plan_id: String,
    #[serde(rename = "_token", default)]
    token: String,
}

pub async fn checkout(user: CurrentUser, session: Session, State(state): State<AppState>, Form(f): Form<CheckoutForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    let back = Redirect::to("/admin/wa/billing").into_response();
    let Ok(pid) = Uuid::parse_str(f.plan_id.trim()) else {
        view::set_flash(&session, "error", "Paket tidak valid.").await;
        return back;
    };
    let plan: Option<(Uuid, String, String, i64, bool)> =
        sqlx::query_as("SELECT id, code, name, price_idr, is_active FROM plans WHERE id=$1")
            .bind(pid).fetch_optional(&state.pool).await.ok().flatten();
    let Some((plan_id, code, name, price, active)) = plan else {
        view::set_flash(&session, "error", "Paket tidak ditemukan.").await;
        return back;
    };
    if !active || price <= 0 {
        view::set_flash(&session, "error", "Paket ini tidak tersedia untuk pembelian.").await;
        return back;
    }
    let s = settings::all(&state.pool).await;
    let cfg = midtrans::MidtransCfg::from_settings(&s);
    if !cfg.is_configured() {
        view::set_flash(&session, "error", "Pembayaran belum dikonfigurasi. Hubungi admin.").await;
        return back;
    }

    let order_id = format!("WA-{}", Uuid::new_v4().simple());
    let inv_id = Uuid::now_v7();
    let _ = sqlx::query(
        "INSERT INTO wa_invoices (id, order_id, user_id, plan_id, plan_code, plan_name, amount_idr, status) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,'pending')",
    )
    .bind(inv_id).bind(&order_id).bind(user.id).bind(plan_id).bind(&code).bind(&name).bind(price)
    .execute(&state.pool).await;

    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://127.0.0.1:8090".into());
    let finish = format!("{app_url}/admin/wa/billing/finish");
    let item_name = format!("Langganan {name} (30 hari)");
    match midtrans::create_snap_transaction(&cfg, &order_id, price, &code, &item_name, &user.name, &user.email, &finish).await {
        Ok((token, redirect)) => {
            let _ = sqlx::query("UPDATE wa_invoices SET snap_token=$2, snap_redirect_url=$3, updated_at=now() WHERE id=$1")
                .bind(inv_id).bind(&token).bind(&redirect).execute(&state.pool).await;
            Redirect::to(&redirect).into_response()
        }
        Err(e) => {
            let _ = sqlx::query("UPDATE wa_invoices SET status='failed', updated_at=now() WHERE id=$1")
                .bind(inv_id).execute(&state.pool).await;
            tracing::warn!("Midtrans checkout gagal order={order_id}: {e}");
            view::set_flash(&session, "error", "Gagal memulai pembayaran. Coba lagi atau hubungi admin.").await;
            Redirect::to("/admin/wa/billing").into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct FinishQ {
    #[serde(default)]
    transaction_status: String,
}

/// Redirect balik dari Snap (feedback UI saja; status SEBENARNYA dari notifikasi server).
pub async fn finish(Query(q): Query<FinishQ>) -> Redirect {
    let st = match q.transaction_status.as_str() {
        "settlement" | "capture" => "success",
        "pending" => "pending",
        "" => "pending",
        _ => "error",
    };
    Redirect::to(&format!("/admin/wa/billing?status={st}"))
}

/// Notifikasi server-to-server Midtrans (PUBLIK, di luar require_auth). Dijaga signature SHA512.
pub async fn midtrans_notification(State(state): State<AppState>, Json(payload): Json<Value>) -> Response {
    let s = settings::all(&state.pool).await;
    let cfg = midtrans::MidtransCfg::from_settings(&s);
    if cfg.server_key.is_empty() {
        return (StatusCode::SERVICE_UNAVAILABLE, "billing not configured").into_response();
    }
    let g = |k: &str| payload.get(k).and_then(|v| v.as_str()).unwrap_or("");
    let order_id = g("order_id");
    let status_code = g("status_code");
    let gross_amount = g("gross_amount");
    let signature = g("signature_key");
    let tx_status = g("transaction_status");
    let fraud = g("fraud_status");
    let pay_type = g("payment_type");

    if !midtrans::verify_signature(&cfg.server_key, order_id, status_code, gross_amount, signature) {
        return (StatusCode::FORBIDDEN, "invalid signature").into_response();
    }

    let inv: Option<(Uuid, Uuid, Uuid, i64, i32, String)> = sqlx::query_as(
        "SELECT id, user_id, plan_id, amount_idr, period_days, status FROM wa_invoices WHERE order_id=$1",
    )
    .bind(order_id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    // order tak dikenal → 200 (jangan picu retry badai)
    let Some((inv_id, user_id, plan_id, amount_idr, period_days, cur_status)) = inv else {
        return (StatusCode::OK, "ignored").into_response();
    };
    if cur_status == "paid" {
        return (StatusCode::OK, "already paid").into_response();
    }
    let raw = payload.to_string();
    let paid = tx_status == "settlement" || (tx_status == "capture" && fraud == "accept");

    if paid {
        // Anti-tamper: harga dari DB, cross-check gross_amount notif.
        let notif_amount: i64 = gross_amount.split('.').next().unwrap_or("0").parse().unwrap_or(0);
        if notif_amount != amount_idr {
            let _ = sqlx::query("UPDATE wa_invoices SET midtrans_transaction_status=$2, midtrans_payment_type=$3, raw_notification=$4::jsonb, updated_at=now() WHERE id=$1")
                .bind(inv_id).bind(tx_status).bind(pay_type).bind(&raw).execute(&state.pool).await;
            tracing::warn!("Midtrans amount mismatch order={order_id} notif={notif_amount} db={amount_idr} — TIDAK diaktifkan");
            return (StatusCode::OK, "amount mismatch").into_response();
        }
        match state.pool.begin().await {
            Ok(mut tx) => {
                let upd = sqlx::query(
                    "UPDATE wa_invoices SET status='paid', paid_at=now(), midtrans_transaction_status=$2, \
                     midtrans_payment_type=$3, raw_notification=$4::jsonb, updated_at=now() \
                     WHERE order_id=$1 AND status<>'paid'",
                )
                .bind(order_id).bind(tx_status).bind(pay_type).bind(&raw)
                .execute(&mut *tx).await;
                if upd.map(|r| r.rows_affected()).unwrap_or(0) == 1 {
                    // aktivasi/perpanjang plan: akumulatif (+period_days dari sisa/now)
                    let _ = sqlx::query(
                        "UPDATE users SET plan_id=$2, plan_started_at=now(), \
                         plan_expires_at = GREATEST(COALESCE(plan_expires_at, now()), now()) + make_interval(days => $3::int) \
                         WHERE id=$1",
                    )
                    .bind(user_id).bind(plan_id).bind(period_days)
                    .execute(&mut *tx).await;
                    let _ = sqlx::query("UPDATE wa_invoices SET applied_at=now() WHERE id=$1 AND applied_at IS NULL")
                        .bind(inv_id).execute(&mut *tx).await;
                }
                let _ = tx.commit().await;
                (StatusCode::OK, "ok").into_response()
            }
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response(),
        }
    } else {
        let new_status = match tx_status {
            "deny" | "failure" => "failed",
            "cancel" => "canceled",
            "expire" => "expired",
            _ => "pending",
        };
        let _ = sqlx::query(
            "UPDATE wa_invoices SET status=$2, midtrans_transaction_status=$3, midtrans_payment_type=$4, \
             raw_notification=$5::jsonb, updated_at=now() WHERE order_id=$1 AND status<>'paid'",
        )
        .bind(order_id).bind(new_status).bind(tx_status).bind(pay_type).bind(&raw)
        .execute(&state.pool).await;
        (StatusCode::OK, "ok").into_response()
    }
}
