//! Helper plan & kuota (enforcement SaaS). Superadmin di-bypass di sisi pemanggil.

use sqlx::types::Uuid;
use sqlx::PgPool;

/// (max_sessions, max_messages_per_day, ai_enabled, webhook_enabled) untuk plan user.
/// Default aman (Basic) bila user tak punya plan.
pub async fn plan_limits(pool: &PgPool, user_id: Uuid) -> (i64, i64, bool, bool) {
    // Plan yang SUDAH kadaluarsa dianggap tidak aktif → jatuh ke default (Basic).
    // (Job auto-downgrade membersihkan plan_id-nya; ini jaring pengaman langsung.)
    let row: Option<(i32, i32, bool, bool)> = sqlx::query_as(
        "SELECT p.max_sessions, p.max_messages_per_day, p.ai_enabled, p.webhook_enabled \
         FROM users u JOIN plans p ON p.id = u.plan_id \
         WHERE u.id = $1 AND (u.plan_expires_at IS NULL OR u.plan_expires_at > now())",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    match row {
        Some((s, m, ai, wh)) => (s as i64, m as i64, ai, wh),
        None => (2, 200, false, false),
    }
}

/// Jumlah nomor (sesi) milik user saat ini.
pub async fn session_count(pool: &PgPool, user_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM wa_sessions WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0)
}

/// Total pesan keluar hari ini (gabungan semua sesi user).
pub async fn sent_today(pool: &PgPool, user_id: Uuid) -> i64 {
    sqlx::query_scalar(
        "SELECT COALESCE(sum(d.sent_count),0)::bigint FROM wa_sessions s \
         LEFT JOIN wa_usage_daily d ON d.session_id = s.id AND d.day = CURRENT_DATE \
         WHERE s.user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0)
}

/// UPSERT counter harian. `col` ∈ {sent, received, ai_reply, failed} (SQL statik per kolom).
pub async fn bump(pool: &PgPool, session_id: Uuid, col: &str) {
    let sql = match col {
        "sent" => "INSERT INTO wa_usage_daily (session_id,day,sent_count,updated_at) VALUES ($1,CURRENT_DATE,1,now()) ON CONFLICT (session_id,day) DO UPDATE SET sent_count=wa_usage_daily.sent_count+1, updated_at=now()",
        "received" => "INSERT INTO wa_usage_daily (session_id,day,received_count,updated_at) VALUES ($1,CURRENT_DATE,1,now()) ON CONFLICT (session_id,day) DO UPDATE SET received_count=wa_usage_daily.received_count+1, updated_at=now()",
        "ai_reply" => "INSERT INTO wa_usage_daily (session_id,day,ai_reply_count,updated_at) VALUES ($1,CURRENT_DATE,1,now()) ON CONFLICT (session_id,day) DO UPDATE SET ai_reply_count=wa_usage_daily.ai_reply_count+1, updated_at=now()",
        "failed" => "INSERT INTO wa_usage_daily (session_id,day,failed_count,updated_at) VALUES ($1,CURRENT_DATE,1,now()) ON CONFLICT (session_id,day) DO UPDATE SET failed_count=wa_usage_daily.failed_count+1, updated_at=now()",
        _ => return,
    };
    let _ = sqlx::query(sql).bind(session_id).execute(pool).await;
}
