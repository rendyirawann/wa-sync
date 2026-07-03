//! Dashboard dengan DATA NYATA, di-scope per-user (Superadmin = seluruh platform).
//! Menggantikan handler dummy lama.

use axum::{extract::State, response::Html};
use serde_json::json;
use tower_sessions::Session;

use crate::{rbac::CurrentUser, view, AppState};

pub async fn index(user: CurrentUser, session: Session, State(state): State<AppState>) -> Html<String> {
    let pool = &state.pool;
    let su = user.is_superadmin();
    let uid = user.id;

    macro_rules! count {
        ($all:literal, $own:literal) => {
            if su {
                sqlx::query_scalar::<_, i64>($all).fetch_one(pool).await.unwrap_or(0)
            } else {
                sqlx::query_scalar::<_, i64>($own).bind(uid).fetch_one(pool).await.unwrap_or(0)
            }
        };
    }

    let sess_total: i64 = count!(
        "SELECT count(*) FROM wa_sessions",
        "SELECT count(*) FROM wa_sessions WHERE user_id=$1"
    );
    let sess_conn: i64 = count!(
        "SELECT count(*) FROM wa_sessions WHERE status='connected'",
        "SELECT count(*) FROM wa_sessions WHERE user_id=$1 AND status='connected'"
    );
    let msg_out: i64 = count!(
        "SELECT count(*) FROM wa_messages WHERE direction='out' AND created_at > now() - interval '30 days'",
        "SELECT count(*) FROM wa_messages m JOIN wa_sessions s ON m.session_id=s.id WHERE s.user_id=$1 AND m.direction='out' AND m.created_at > now() - interval '30 days'"
    );
    let msg_in: i64 = count!(
        "SELECT count(*) FROM wa_messages WHERE direction='in' AND created_at > now() - interval '30 days'",
        "SELECT count(*) FROM wa_messages m JOIN wa_sessions s ON m.session_id=s.id WHERE s.user_id=$1 AND m.direction='in' AND m.created_at > now() - interval '30 days'"
    );
    let contacts: i64 = count!(
        "SELECT count(*) FROM users",
        "SELECT count(DISTINCT m.remote_jid) FROM wa_messages m JOIN wa_sessions s ON m.session_id=s.id WHERE s.user_id=$1"
    );

    // Sesi terbaru (max 6)
    let srows: Vec<(String, Option<String>, String, Option<String>)> = if su {
        sqlx::query_as("SELECT label, phone, status, to_char(last_connected_at AT TIME ZONE 'Asia/Jakarta','DD Mon HH24:MI') FROM wa_sessions ORDER BY created_at DESC LIMIT 6")
            .fetch_all(pool).await.unwrap_or_default()
    } else {
        sqlx::query_as("SELECT label, phone, status, to_char(last_connected_at AT TIME ZONE 'Asia/Jakarta','DD Mon HH24:MI') FROM wa_sessions WHERE user_id=$1 ORDER BY created_at DESC LIMIT 6")
            .bind(uid).fetch_all(pool).await.unwrap_or_default()
    };
    let sessions: Vec<_> = srows.iter().map(|r| json!({
        "label": r.0, "phone": r.1, "status": r.2, "last": r.3.clone().unwrap_or_else(|| "—".into())
    })).collect();

    // Aktivitas (pesan terbaru, max 6)
    let mrows: Vec<(String, String, Option<String>, Option<String>)> = if su {
        sqlx::query_as("SELECT direction, remote_jid, body, to_char(created_at AT TIME ZONE 'Asia/Jakarta','DD Mon HH24:MI') FROM wa_messages ORDER BY created_at DESC LIMIT 6")
            .fetch_all(pool).await.unwrap_or_default()
    } else {
        sqlx::query_as("SELECT m.direction, m.remote_jid, m.body, to_char(m.created_at AT TIME ZONE 'Asia/Jakarta','DD Mon HH24:MI') FROM wa_messages m JOIN wa_sessions s ON m.session_id=s.id WHERE s.user_id=$1 ORDER BY m.created_at DESC LIMIT 6")
            .bind(uid).fetch_all(pool).await.unwrap_or_default()
    };
    let activity: Vec<_> = mrows.iter().map(|r| json!({
        "direction": r.0,
        "jid": r.1.split('@').next().unwrap_or(&r.1),
        "body": r.2.clone().unwrap_or_default(),
        "when": r.3.clone().unwrap_or_default()
    })).collect();

    let mut ctx = view::base_context(&state, &session, &user, "dashboard").await;
    ctx.insert("sess_total", &sess_total);
    ctx.insert("sess_conn", &sess_conn);
    ctx.insert("msg_out", &msg_out);
    ctx.insert("msg_in", &msg_in);
    ctx.insert("contacts", &contacts);
    ctx.insert("dash_sessions", &sessions);
    ctx.insert("dash_activity", &activity);
    match state.tera.render("dashboard.html", &ctx) {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}
