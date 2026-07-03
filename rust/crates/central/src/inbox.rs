//! Inbox WhatsApp (P3c): daftar percakapan + thread + balas. Scoped per-user
//! (Superadmin lihat semua). Balasan dikirim lewat antrian throttle sidecar.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Form, Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::types::Uuid;
use std::collections::HashMap;
use tower_sessions::Session;

use crate::{auth, rbac::CurrentUser, view, AppState};

const CONV_ALL: &str = "SELECT DISTINCT ON (m.session_id, m.remote_jid) m.session_id, m.remote_jid, m.body, \
    to_char(m.created_at AT TIME ZONE 'Asia/Jakarta','DD Mon HH24:MI'), s.label, extract(epoch from m.created_at)::bigint, COALESCE(c.name,''), COALESCE(c.ai_persona,''), COALESCE(m.msg_type,'text'), COALESCE(c.ai_paused,false) \
    FROM wa_messages m JOIN wa_sessions s ON m.session_id = s.id \
    LEFT JOIN wa_contacts c ON c.session_id = m.session_id AND c.jid = m.remote_jid \
    ORDER BY m.session_id, m.remote_jid, m.created_at DESC";
const CONV_OWN: &str = "SELECT DISTINCT ON (m.session_id, m.remote_jid) m.session_id, m.remote_jid, m.body, \
    to_char(m.created_at AT TIME ZONE 'Asia/Jakarta','DD Mon HH24:MI'), s.label, extract(epoch from m.created_at)::bigint, COALESCE(c.name,''), COALESCE(c.ai_persona,''), COALESCE(m.msg_type,'text'), COALESCE(c.ai_paused,false) \
    FROM wa_messages m JOIN wa_sessions s ON m.session_id = s.id \
    LEFT JOIN wa_contacts c ON c.session_id = m.session_id AND c.jid = m.remote_jid \
    WHERE s.user_id = $1 \
    ORDER BY m.session_id, m.remote_jid, m.created_at DESC";

/// Label ringkas pesan terakhir (untuk daftar percakapan): pakai body, atau ikon media bila kosong.
fn last_label(body: &str, mtype: &str) -> String {
    if !body.trim().is_empty() {
        return body.to_string();
    }
    match mtype {
        "image" => "📷 Foto",
        "video" => "🎬 Video",
        "document" => "📄 Dokumen",
        "audio" => "🎵 Pesan suara",
        "sticker" => "🌟 Stiker",
        _ => "",
    }
    .to_string()
}

async fn owns_session(state: &AppState, user: &CurrentUser, sid: Uuid) -> bool {
    let found: Option<(Uuid,)> = if user.is_superadmin() {
        sqlx::query_as("SELECT id FROM wa_sessions WHERE id=$1").bind(sid)
            .fetch_optional(&state.pool).await.ok().flatten()
    } else {
        sqlx::query_as("SELECT id FROM wa_sessions WHERE id=$1 AND user_id=$2").bind(sid).bind(user.id)
            .fetch_optional(&state.pool).await.ok().flatten()
    };
    found.is_some()
}

pub async fn index(user: CurrentUser, session: Session, State(state): State<AppState>) -> Html<String> {
    let mut rows: Vec<(Uuid, String, Option<String>, Option<String>, String, i64, String, String, String, bool)> = if user.is_superadmin() {
        sqlx::query_as(CONV_ALL).fetch_all(&state.pool).await.unwrap_or_default()
    } else {
        sqlx::query_as(CONV_OWN).bind(user.id).fetch_all(&state.pool).await.unwrap_or_default()
    };
    rows.sort_by(|a, b| b.5.cmp(&a.5)); // terbaru dulu
    let convs: Vec<_> = rows.iter().map(|r| {
        let phone = r.1.split('@').next().unwrap_or(&r.1).to_string();
        json!({
            "session_id": r.0.to_string(),
            "jid": r.1,
            "phone": phone,
            "name": r.6,
            "persona": r.7,
            "last": last_label(&r.2.clone().unwrap_or_default(), &r.8),
            "when": r.3.clone().unwrap_or_default(),
            "session_label": r.4,
            "ai_paused": r.9,
        })
    }).collect();

    let mut ctx = view::base_context(&state, &session, &user, "messages").await;
    ctx.insert("conversations", &convs);
    match state.tera.render("wa/inbox.html", &ctx) {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

/// JSON thread satu percakapan: /admin/wa/messages/thread?session=<uuid>&jid=<jid>
pub async fn thread(user: CurrentUser, State(state): State<AppState>, Query(q): Query<HashMap<String, String>>) -> Response {
    let Some(sid) = q.get("session").and_then(|s| Uuid::parse_str(s).ok()) else {
        return Json(json!({ "messages": [] })).into_response();
    };
    if !owns_session(&state, &user, sid).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let jid = q.get("jid").cloned().unwrap_or_default();
    let rows: Vec<(String, Option<String>, String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT direction, body, to_char(created_at AT TIME ZONE 'Asia/Jakarta','DD Mon HH24:MI'), status, COALESCE(msg_type,'text'), media_url \
         FROM wa_messages WHERE session_id=$1 AND remote_jid=$2 ORDER BY created_at ASC LIMIT 300",
    )
    .bind(sid).bind(&jid).fetch_all(&state.pool).await.unwrap_or_default();
    let msgs: Vec<_> = rows.iter().map(|r| json!({
        "direction": r.0, "body": r.1.clone().unwrap_or_default(), "when": r.2, "status": r.3,
        "type": r.4, "media": r.5.clone().unwrap_or_default()
    })).collect();
    Json(json!({ "messages": msgs })).into_response()
}

#[derive(Deserialize)]
pub struct SendForm {
    session_id: String,
    to: String,
    #[serde(default)]
    text: String,
    #[serde(rename = "_token", default)]
    token: String,
}

/// Balas pesan → enqueue ke sidecar (lewat throttle anti-ban).
pub async fn send(user: CurrentUser, session: Session, State(state): State<AppState>, Form(f): Form<SendForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    let Ok(sid) = Uuid::parse_str(&f.session_id) else {
        return (StatusCode::BAD_REQUEST, "bad session").into_response();
    };
    if !owns_session(&state, &user, sid).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    if f.text.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "status": "error", "message": "pesan kosong" }))).into_response();
    }
    // Enforcement kuota pesan harian per-plan (Superadmin bypass).
    if !user.is_superadmin() {
        let (_, max_msg, _, _) = crate::quota::plan_limits(&state.pool, user.id).await;
        if max_msg > 0 && crate::quota::sent_today(&state.pool, user.id).await >= max_msg {
            return (StatusCode::TOO_MANY_REQUESTS, Json(json!({ "status": "error", "message": "Kuota pesan harian habis. Upgrade plan." }))).into_response();
        }
    }
    // Human handoff: agen membalas manual → jeda AI untuk kontak ini (bisa diaktifkan lagi via toggle).
    let _ = sqlx::query(
        "INSERT INTO wa_contacts (session_id, jid, ai_paused, updated_at) VALUES ($1,$2,true,now()) \
         ON CONFLICT (session_id, jid) DO UPDATE SET ai_paused=true, updated_at=now()",
    )
    .bind(sid).bind(&f.to).execute(&state.pool).await;
    match state.wa.send(&sid.to_string(), &f.to, &f.text).await {
        Ok(_) => Json(json!({ "status": "queued", "ai_paused": true })).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, Json(json!({ "status": "error", "message": e.to_string() }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct AiToggleForm {
    session_id: String,
    jid: String,
    #[serde(default)]
    pause: String,
    #[serde(rename = "_token", default)]
    token: String,
}

/// Jeda / lanjutkan AI untuk satu percakapan (human handoff).
pub async fn ai_toggle(user: CurrentUser, session: Session, State(state): State<AppState>, Form(f): Form<AiToggleForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    let Ok(sid) = Uuid::parse_str(&f.session_id) else {
        return (StatusCode::BAD_REQUEST, "bad session").into_response();
    };
    if !owns_session(&state, &user, sid).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let paused = matches!(f.pause.as_str(), "1" | "true" | "on");
    let _ = sqlx::query(
        "INSERT INTO wa_contacts (session_id, jid, ai_paused, updated_at) VALUES ($1,$2,$3,now()) \
         ON CONFLICT (session_id, jid) DO UPDATE SET ai_paused=EXCLUDED.ai_paused, updated_at=now()",
    )
    .bind(sid).bind(f.jid.trim()).bind(paused).execute(&state.pool).await;
    Json(json!({ "status": "ok", "ai_paused": paused })).into_response()
}

#[derive(Deserialize)]
pub struct ResendForm {
    session_id: String,
    jid: String,
    #[serde(rename = "_token", default)]
    token: String,
}

/// Kirim ULANG balasan terakhir (pesan keluar terakhir) ke kontak ini — berguna bila balasan
/// sebelumnya tak sampai (mis. nomor @lid yang kini sudah bisa diresolusi ke nomor asli).
pub async fn resend(user: CurrentUser, session: Session, State(state): State<AppState>, Form(f): Form<ResendForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    let Ok(sid) = Uuid::parse_str(&f.session_id) else {
        return (StatusCode::BAD_REQUEST, "bad session").into_response();
    };
    if !owns_session(&state, &user, sid).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let last: Option<(String,)> = sqlx::query_as(
        "SELECT body FROM wa_messages WHERE session_id=$1 AND remote_jid=$2 AND direction='out' AND body IS NOT NULL \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(sid).bind(&f.jid).fetch_optional(&state.pool).await.ok().flatten();
    let Some((body,)) = last else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "status": "error", "message": "tidak ada balasan keluar untuk dikirim ulang" }))).into_response();
    };
    match state.wa.send(&sid.to_string(), &f.jid, &body).await {
        Ok(_) => Json(json!({ "status": "queued" })).into_response(),
        Err(e) => (StatusCode::BAD_GATEWAY, Json(json!({ "status": "error", "message": e.to_string() }))).into_response(),
    }
}

#[derive(Deserialize)]
pub struct ContactForm {
    session_id: String,
    jid: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    ai_persona: String,
    #[serde(rename = "_token", default)]
    token: String,
}

/// Simpan nama + persona AI per-kontak. Mis. nama "Suci" + persona "balas sebagai pacar".
pub async fn set_contact(user: CurrentUser, session: Session, State(state): State<AppState>, Form(f): Form<ContactForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    let Ok(sid) = Uuid::parse_str(&f.session_id) else {
        return (StatusCode::BAD_REQUEST, "bad session").into_response();
    };
    if !owns_session(&state, &user, sid).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let name = f.name.trim();
    let persona = f.ai_persona.trim();
    let _ = sqlx::query(
        "INSERT INTO wa_contacts (session_id, jid, name, ai_persona, updated_at) VALUES ($1,$2,$3,$4,now()) \
         ON CONFLICT (session_id, jid) DO UPDATE SET name=EXCLUDED.name, ai_persona=EXCLUDED.ai_persona, updated_at=now()",
    )
    .bind(sid).bind(&f.jid).bind(name).bind(persona).execute(&state.pool).await;
    Json(json!({ "status": "ok", "name": name, "persona": persona })).into_response()
}
