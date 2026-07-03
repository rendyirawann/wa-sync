use chrono::{Duration, Local, NaiveDateTime};
use sqlx::types::Uuid;
use sqlx::PgPool;

/// Hitung expired_at dari durasi ban (None = permanen).
fn expiry_from(duration: &str) -> Option<NaiveDateTime> {
    let now = Local::now().naive_local();
    match duration {
        "1h" => Some(now + Duration::hours(1)),
        "24h" => Some(now + Duration::days(1)),
        "1w" => Some(now + Duration::weeks(1)),
        _ => None, // permanent
    }
}

/// Ban user: catat baris di tabel `bans` + set `users.banned_at`.
pub async fn ban(pool: &PgPool, user_id: Uuid, by: Uuid, reason: &str, duration: &str) -> anyhow::Result<()> {
    let expired = expiry_from(duration);
    sqlx::query(
        "INSERT INTO bans (bannable_type, bannable_id, created_by_type, created_by_id, comment, expired_at, created_at, updated_at) \
         VALUES ('App\\Models\\User', $1, 'App\\Models\\User', $2, $3, $4, now(), now())",
    )
    .bind(user_id)
    .bind(by)
    .bind(reason)
    .bind(expired)
    .execute(pool)
    .await?;
    sqlx::query("UPDATE users SET banned_at = now() WHERE id = $1").bind(user_id).execute(pool).await?;
    Ok(())
}

/// Unban: soft-delete ban aktif + kosongkan `users.banned_at`.
pub async fn unban(pool: &PgPool, user_id: Uuid) -> anyhow::Result<()> {
    sqlx::query("UPDATE bans SET deleted_at = now() WHERE bannable_id = $1 AND deleted_at IS NULL")
        .bind(user_id).execute(pool).await?;
    sqlx::query("UPDATE users SET banned_at = NULL WHERE id = $1").bind(user_id).execute(pool).await?;
    Ok(())
}

/// Apakah user sedang dibanned (flag `banned_at`).
pub fn is_banned(banned_at: Option<NaiveDateTime>) -> bool {
    banned_at.is_some()
}

/// Badge status HTML: Active / Banned (permanen) / Suspended until X / Ban Expired.
pub fn status_html(banned_at: Option<NaiveDateTime>, expired_at: Option<NaiveDateTime>) -> String {
    if banned_at.is_none() {
        return "<span class=\"badge bg-success\">Active</span>".into();
    }
    match expired_at {
        None => "<span class=\"badge bg-danger\">Banned</span>".into(),
        Some(exp) => {
            let now = Local::now().naive_local();
            if now > exp {
                "<span class=\"badge bg-warning text-dark\">Ban Expired</span>".into()
            } else {
                format!(
                    "<span class=\"badge bg-warning text-dark\">Suspended until {}</span>",
                    crate::activity::format_id_datetime(exp)
                )
            }
        }
    }
}
