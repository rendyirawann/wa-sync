//! Auto-reply KEYWORD (rule-based) + orkestrasi balasan otomatis pesan masuk.
//! Alur saat pesan masuk: cek aturan keyword dulu → kalau cocok balas & selesai;
//! kalau tidak → lanjut ke AI (ai_pipeline). Juga halaman kelola aturan (per-nomor).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Form, Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::types::Uuid;
use tower_sessions::Session;

use crate::{auth, rbac::CurrentUser, view, AppState};

// ===================== Runtime: balas otomatis =====================

/// Dipanggil (spawn) dari wainternal saat pesan masuk. Keyword dulu, lalu AI.
pub async fn handle(state: AppState, session_id: Uuid, remote_jid: String, incoming: String) {
    if remote_jid.ends_with("@g.us") {
        return;
    }
    // 1) Aturan keyword (rule-based) — tak butuh AI/plan.
    let rules: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT keyword, match_type, reply FROM wa_autoreply \
         WHERE session_id=$1 AND enabled AND reply <> '' ORDER BY sort_order, created_at",
    )
    .bind(session_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let text = incoming.trim().to_lowercase();
    if !text.is_empty() {
        for (kw, mt, reply) in &rules {
            let k = kw.trim().to_lowercase();
            if k.is_empty() {
                continue;
            }
            let hit = match mt.as_str() {
                "exact" => text == k,
                "starts" => text.starts_with(&k),
                _ => text.contains(&k),
            };
            if hit {
                // kirim ke JID asli (sidecar resolusi @lid). Bukan via_ai (rule-based).
                let _ = state.wa.send(&session_id.to_string(), &remote_jid, reply).await;
                return; // aturan cocok → jangan lanjut AI
            }
        }
    }
    // 2) AI fallback (cek ai_enabled/plan/pause/jam/idle di dalam).
    crate::ai_pipeline::maybe_reply(state, session_id, remote_jid).await;
}

// ===================== Halaman kelola aturan =====================

pub async fn index(user: CurrentUser, session: Session, State(state): State<AppState>) -> Html<String> {
    // Sesi milik user (untuk dropdown + grup aturan).
    type S = (Uuid, String, Option<String>);
    let sessions: Vec<S> = if user.is_superadmin() {
        sqlx::query_as("SELECT id, label, phone FROM wa_sessions ORDER BY label").fetch_all(&state.pool).await.unwrap_or_default()
    } else {
        sqlx::query_as("SELECT id, label, phone FROM wa_sessions WHERE user_id=$1 ORDER BY label").bind(user.id).fetch_all(&state.pool).await.unwrap_or_default()
    };
    let sess: Vec<_> = sessions.iter().map(|s| json!({
        "id": s.0.to_string(), "label": s.1, "phone": s.2.clone().unwrap_or_default()
    })).collect();

    type R = (Uuid, Uuid, String, String, String, bool, String);
    let rows: Vec<R> = if user.is_superadmin() {
        sqlx::query_as("SELECT r.id, r.session_id, r.keyword, r.match_type, r.reply, r.enabled, COALESCE(s.label,'') \
             FROM wa_autoreply r JOIN wa_sessions s ON s.id=r.session_id ORDER BY s.label, r.sort_order, r.created_at")
            .fetch_all(&state.pool).await.unwrap_or_default()
    } else {
        sqlx::query_as("SELECT r.id, r.session_id, r.keyword, r.match_type, r.reply, r.enabled, COALESCE(s.label,'') \
             FROM wa_autoreply r JOIN wa_sessions s ON s.id=r.session_id WHERE s.user_id=$1 ORDER BY s.label, r.sort_order, r.created_at")
            .bind(user.id).fetch_all(&state.pool).await.unwrap_or_default()
    };
    let rules: Vec<_> = rows.iter().map(|r| json!({
        "id": r.0.to_string(), "session_id": r.1.to_string(), "keyword": r.2,
        "match_type": r.3, "reply": r.4, "enabled": r.5, "session_label": r.6
    })).collect();

    let mut ctx = view::base_context(&state, &session, &user, "autoreply").await;
    ctx.insert("sessions", &sess);
    ctx.insert("rules", &rules);
    match state.tera.render("wa/autoreply.html", &ctx) {
        Ok(h) => Html(h),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

async fn owns_session(state: &AppState, user: &CurrentUser, sid: Uuid) -> bool {
    let f: Option<(Uuid,)> = if user.is_superadmin() {
        sqlx::query_as("SELECT id FROM wa_sessions WHERE id=$1").bind(sid).fetch_optional(&state.pool).await.ok().flatten()
    } else {
        sqlx::query_as("SELECT id FROM wa_sessions WHERE id=$1 AND user_id=$2").bind(sid).bind(user.id).fetch_optional(&state.pool).await.ok().flatten()
    };
    f.is_some()
}

#[derive(Deserialize)]
pub struct RuleForm {
    #[serde(default)]
    session_id: String,
    #[serde(default)]
    keyword: String,
    #[serde(default)]
    match_type: String,
    #[serde(default)]
    reply: String,
    #[serde(rename = "_token", default)]
    token: String,
}

pub async fn create(user: CurrentUser, session: Session, State(state): State<AppState>, Form(f): Form<RuleForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    let Ok(sid) = Uuid::parse_str(f.session_id.trim()) else {
        view::set_flash(&session, "error", "Pilih nomor.").await;
        return Redirect::to("/admin/wa/autoreply").into_response();
    };
    if !owns_session(&state, &user, sid).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let kw = f.keyword.trim();
    let reply = f.reply.trim();
    if kw.is_empty() || reply.is_empty() {
        view::set_flash(&session, "error", "Keyword & balasan wajib diisi.").await;
        return Redirect::to("/admin/wa/autoreply").into_response();
    }
    let mt = match f.match_type.as_str() {
        "exact" | "starts" | "contains" => f.match_type.as_str(),
        _ => "contains",
    };
    let _ = sqlx::query("INSERT INTO wa_autoreply (id, session_id, keyword, match_type, reply) VALUES ($1,$2,$3,$4,$5)")
        .bind(Uuid::now_v7()).bind(sid).bind(kw).bind(mt).bind(reply).execute(&state.pool).await;
    view::set_flash(&session, "success", "Aturan auto-reply ditambahkan.").await;
    Redirect::to("/admin/wa/autoreply").into_response()
}

pub async fn destroy(user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    let res = if user.is_superadmin() {
        sqlx::query("DELETE FROM wa_autoreply WHERE id=$1").bind(id).execute(&state.pool).await
    } else {
        sqlx::query("DELETE FROM wa_autoreply r USING wa_sessions s WHERE r.id=$1 AND r.session_id=s.id AND s.user_id=$2")
            .bind(id).bind(user.id).execute(&state.pool).await
    };
    match res {
        Ok(_) => Json(json!({ "status": "ok" })).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response(),
    }
}
