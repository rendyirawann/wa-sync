//! Public API (kompatibel pola saungwa): kirim pesan pakai app_key (per-user) + auth_key (per-nomor).
//! Di LUAR require_auth — autentik via kunci. Menerima multipart / urlencoded / JSON.

use axum::{
    body::Bytes,
    extract::{FromRequest, Multipart, Query, Request, State},
    http::{header::CONTENT_TYPE, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::types::Uuid;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use crate::{quota, AppState};

/// Rate-limit per appkey: 60 permintaan / menit (in-memory sliding window).
static API_RL: LazyLock<Mutex<HashMap<String, (Instant, u32)>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
fn rate_ok(key: &str) -> bool {
    const LIMIT: u32 = 60;
    let mut m = API_RL.lock().unwrap();
    let now = Instant::now();
    let e = m.entry(key.to_string()).or_insert((now, 0));
    if now.duration_since(e.0) > Duration::from_secs(60) {
        *e = (now, 0);
    }
    e.1 += 1;
    e.1 <= LIMIT
}

/// Resolusi appkey+authkey → (session_id, user_id) + rate-limit. Err(Response) bila gagal.
async fn auth_session(state: &AppState, appkey: &str, authkey: &str) -> Result<(Uuid, Uuid), Response> {
    let Ok(app) = Uuid::parse_str(appkey.trim()) else {
        return Err(err("invalid_key", "appkey/authkey tidak valid", StatusCode::UNAUTHORIZED));
    };
    if !rate_ok(appkey.trim()) {
        return Err(err("rate_limited", "terlalu banyak permintaan (maks 60/menit)", StatusCode::TOO_MANY_REQUESTS));
    }
    let row: Option<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT s.id, s.user_id FROM wa_sessions s JOIN users u ON u.id=s.user_id WHERE s.auth_key=$1 AND u.app_key=$2",
    )
    .bind(authkey.trim())
    .bind(app)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    row.ok_or_else(|| err("invalid_key", "appkey/authkey tidak valid", StatusCode::UNAUTHORIZED))
}

#[derive(Deserialize, Default)]
pub struct ApiMsg {
    #[serde(default)] pub appkey: String,
    #[serde(default)] pub authkey: String,
    #[serde(default)] pub to: String,
    #[serde(default)] pub message: String,
    /// URL media (gambar/dokumen/video) — dipakai endpoint create-message-media.
    #[serde(default)] pub media: String,
    /// Jenis media: image | video | document. Kosong = ditebak dari ekstensi URL.
    #[serde(default, rename = "type")] pub media_type: String,
    /// Lokasi (endpoint send-location).
    #[serde(default)] pub lat: String,
    #[serde(default)] pub lng: String,
    #[serde(default)] pub name: String,
    #[serde(default)] pub sandbox: Option<String>,
}

// Terima multipart/form-data (pola PHP saungwa), x-www-form-urlencoded, atau JSON.
impl<S: Send + Sync> FromRequest<S> for ApiMsg {
    type Rejection = Response;
    async fn from_request(req: Request, state: &S) -> Result<Self, Response> {
        let ct = req.headers().get(CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
        if ct.starts_with("multipart/form-data") {
            let mut mp = Multipart::from_request(req, state).await.map_err(|e| e.into_response())?;
            let mut m: HashMap<String, String> = HashMap::new();
            while let Ok(Some(field)) = mp.next_field().await {
                let name = field.name().unwrap_or("").to_string();
                let val = field.text().await.unwrap_or_default();
                if !name.is_empty() {
                    m.insert(name, val);
                }
            }
            Ok(ApiMsg {
                appkey: m.remove("appkey").unwrap_or_default(),
                authkey: m.remove("authkey").unwrap_or_default(),
                to: m.remove("to").unwrap_or_default(),
                message: m.remove("message").unwrap_or_default(),
                media: m.remove("media").or_else(|| m.remove("media_url")).unwrap_or_default(),
                media_type: m.remove("type").unwrap_or_default(),
                lat: m.remove("lat").unwrap_or_default(),
                lng: m.remove("lng").unwrap_or_default(),
                name: m.remove("name").unwrap_or_default(),
                sandbox: m.remove("sandbox"),
            })
        } else {
            let bytes = Bytes::from_request(req, state).await.map_err(|e| e.into_response())?;
            let parsed = if ct.contains("application/json") {
                serde_json::from_slice::<ApiMsg>(&bytes).ok()
            } else {
                serde_urlencoded::from_bytes::<ApiMsg>(&bytes).ok()
            };
            parsed.ok_or_else(|| {
                (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"status":false,"message":"body tidak valid","code":"validation"}))).into_response()
            })
        }
    }
}

fn err(code: &str, msg: &str, sc: StatusCode) -> Response {
    (sc, Json(json!({ "status": false, "message": msg, "code": code }))).into_response()
}

/// POST /api/create-message — kirim pesan via antrian throttle (anti-ban) + cek kuota.
pub async fn create_message(State(state): State<AppState>, msg: ApiMsg) -> Response {
    let Ok(appkey) = Uuid::parse_str(msg.appkey.trim()) else {
        return err("invalid_key", "appkey/authkey tidak valid", StatusCode::UNAUTHORIZED);
    };
    if !rate_ok(msg.appkey.trim()) {
        return err("rate_limited", "terlalu banyak permintaan (maks 60/menit)", StatusCode::TOO_MANY_REQUESTS);
    }
    let row: Option<(Uuid, String, Uuid)> = sqlx::query_as(
        "SELECT s.id, s.status, s.user_id FROM wa_sessions s JOIN users u ON u.id = s.user_id \
         WHERE s.auth_key = $1 AND u.app_key = $2",
    )
    .bind(msg.authkey.trim())
    .bind(appkey)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    let Some((sid, _status, uid)) = row else {
        return err("invalid_key", "appkey/authkey tidak valid", StatusCode::UNAUTHORIZED);
    };
    let to = msg.to.trim();
    let text = msg.message.trim();
    if to.is_empty() || text.is_empty() {
        return err("validation", "'to' dan 'message' wajib diisi", StatusCode::UNPROCESSABLE_ENTITY);
    }
    let (_, max_msg, _, _) = quota::plan_limits(&state.pool, uid).await;
    if max_msg > 0 && quota::sent_today(&state.pool, uid).await >= max_msg {
        return err("quota_exceeded", "Kuota pesan harian habis. Upgrade plan.", StatusCode::FORBIDDEN);
    }
    if msg.sandbox.as_deref() == Some("true") {
        return (StatusCode::OK, Json(json!({"status":true,"message_status":"Success","message":"sandbox: pesan tidak benar-benar dikirim","id":sid.to_string(),"data":{"to":to}}))).into_response();
    }
    match state.wa.send(&sid.to_string(), to, text).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status":true,"message_status":"Success","message":"queued","id":sid.to_string(),"data":{"to":to}}))).into_response(),
        Err(e) => err("gateway_error", &format!("gagal kirim ke gateway: {e}"), StatusCode::BAD_GATEWAY),
    }
}

/// Tebak jenis media dari ekstensi URL bila tak diberikan.
fn infer_media_type(url: &str) -> &'static str {
    let u = url.split(['?', '#']).next().unwrap_or(url).to_lowercase();
    if u.ends_with(".jpg") || u.ends_with(".jpeg") || u.ends_with(".png") || u.ends_with(".webp") || u.ends_with(".gif") {
        "image"
    } else if u.ends_with(".mp4") || u.ends_with(".3gp") || u.ends_with(".mov") {
        "video"
    } else {
        "document"
    }
}

/// POST /api/create-message-media — kirim media (gambar/dokumen/video) via URL + caption.
pub async fn create_message_media(State(state): State<AppState>, msg: ApiMsg) -> Response {
    let Ok(appkey) = Uuid::parse_str(msg.appkey.trim()) else {
        return err("invalid_key", "appkey/authkey tidak valid", StatusCode::UNAUTHORIZED);
    };
    if !rate_ok(msg.appkey.trim()) {
        return err("rate_limited", "terlalu banyak permintaan (maks 60/menit)", StatusCode::TOO_MANY_REQUESTS);
    }
    let row: Option<(Uuid, String, Uuid)> = sqlx::query_as(
        "SELECT s.id, s.status, s.user_id FROM wa_sessions s JOIN users u ON u.id = s.user_id \
         WHERE s.auth_key = $1 AND u.app_key = $2",
    )
    .bind(msg.authkey.trim())
    .bind(appkey)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    let Some((sid, _status, uid)) = row else {
        return err("invalid_key", "appkey/authkey tidak valid", StatusCode::UNAUTHORIZED);
    };
    let to = msg.to.trim();
    let media = msg.media.trim();
    if to.is_empty() || media.is_empty() {
        return err("validation", "'to' dan 'media' (URL) wajib diisi", StatusCode::UNPROCESSABLE_ENTITY);
    }
    if !(media.starts_with("http://") || media.starts_with("https://")) {
        return err("validation", "'media' harus berupa URL http(s)", StatusCode::UNPROCESSABLE_ENTITY);
    }
    let mtype = match msg.media_type.trim() {
        "image" | "video" | "document" => msg.media_type.trim(),
        _ => infer_media_type(media),
    };
    let caption = msg.message.trim();

    let (_, max_msg, _, _) = quota::plan_limits(&state.pool, uid).await;
    if max_msg > 0 && quota::sent_today(&state.pool, uid).await >= max_msg {
        return err("quota_exceeded", "Kuota pesan harian habis. Upgrade plan.", StatusCode::FORBIDDEN);
    }
    if msg.sandbox.as_deref() == Some("true") {
        return (StatusCode::OK, Json(json!({"status":true,"message_status":"Success","message":"sandbox: media tidak benar-benar dikirim","id":sid.to_string(),"data":{"to":to,"type":mtype}}))).into_response();
    }
    match state.wa.send_media(&sid.to_string(), to, media, mtype, caption).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status":true,"message_status":"Success","message":"queued","id":sid.to_string(),"data":{"to":to,"type":mtype}}))).into_response(),
        Err(e) => err("gateway_error", &format!("gagal kirim ke gateway: {e}"), StatusCode::BAD_GATEWAY),
    }
}

/// GET /api/status?appkey=..&authkey=.. — status nomor + sisa kuota.
pub async fn status(State(state): State<AppState>, Query(q): Query<HashMap<String, String>>) -> Response {
    let appkey_s = q.get("appkey").cloned().unwrap_or_default();
    let authkey = q.get("authkey").cloned().unwrap_or_default();
    let Ok(appkey) = Uuid::parse_str(appkey_s.trim()) else {
        return err("invalid_key", "appkey/authkey tidak valid", StatusCode::UNAUTHORIZED);
    };
    let row: Option<(String, Uuid, String)> = sqlx::query_as(
        "SELECT s.status, s.user_id, COALESCE(s.phone,'') FROM wa_sessions s JOIN users u ON u.id = s.user_id \
         WHERE s.auth_key = $1 AND u.app_key = $2",
    )
    .bind(authkey.trim())
    .bind(appkey)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    let Some((st, uid, phone)) = row else {
        return err("invalid_key", "appkey/authkey tidak valid", StatusCode::UNAUTHORIZED);
    };
    let (_, max_msg, _, _) = quota::plan_limits(&state.pool, uid).await;
    let used = quota::sent_today(&state.pool, uid).await;
    Json(json!({
        "status": true,
        "session_status": st,
        "phone": phone,
        "quota": { "used_today": used, "max_per_day": max_msg }
    })).into_response()
}

/// GET /api/messages?appkey=&authkey=&jid=&limit= — riwayat pesan (opsional per-kontak).
pub async fn get_messages(State(state): State<AppState>, Query(q): Query<HashMap<String, String>>) -> Response {
    let (sid, _uid) = match auth_session(&state, q.get("appkey").map(String::as_str).unwrap_or(""), q.get("authkey").map(String::as_str).unwrap_or("")).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let jid = q.get("jid").cloned().unwrap_or_default();
    let limit: i64 = q.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50).clamp(1, 200);
    type Row = (String, String, Option<String>, String, Option<String>, String, chrono::NaiveDateTime);
    let rows: Vec<Row> = if jid.trim().is_empty() {
        sqlx::query_as("SELECT direction, remote_jid, body, COALESCE(msg_type,'text'), media_url, status, created_at::timestamp FROM wa_messages WHERE session_id=$1 ORDER BY created_at DESC LIMIT $2")
            .bind(sid).bind(limit).fetch_all(&state.pool).await.unwrap_or_default()
    } else {
        sqlx::query_as("SELECT direction, remote_jid, body, COALESCE(msg_type,'text'), media_url, status, created_at::timestamp FROM wa_messages WHERE session_id=$1 AND remote_jid=$2 ORDER BY created_at DESC LIMIT $3")
            .bind(sid).bind(jid.trim()).bind(limit).fetch_all(&state.pool).await.unwrap_or_default()
    };
    let data: Vec<_> = rows.iter().map(|r| json!({
        "direction": r.0, "from": r.1, "body": r.2.clone().unwrap_or_default(),
        "type": r.3, "media": r.4.clone().unwrap_or_default(), "status": r.5, "at": r.6.to_string()
    })).collect();
    Json(json!({ "status": true, "data": data })).into_response()
}

/// GET /api/contacts?appkey=&authkey= — daftar kontak (nama + tag).
pub async fn get_contacts(State(state): State<AppState>, Query(q): Query<HashMap<String, String>>) -> Response {
    let (sid, _uid) = match auth_session(&state, q.get("appkey").map(String::as_str).unwrap_or(""), q.get("authkey").map(String::as_str).unwrap_or("")).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let rows: Vec<(String, String, String)> = sqlx::query_as("SELECT jid, COALESCE(name,''), COALESCE(tags,'') FROM wa_contacts WHERE session_id=$1 ORDER BY name")
        .bind(sid).fetch_all(&state.pool).await.unwrap_or_default();
    let data: Vec<_> = rows.iter().map(|r| {
        let tags: Vec<&str> = r.2.split(',').map(|t| t.trim()).filter(|t| !t.is_empty()).collect();
        json!({ "phone": r.0.split('@').next().unwrap_or(&r.0), "jid": r.0, "name": r.1, "tags": tags })
    }).collect();
    Json(json!({ "status": true, "data": data })).into_response()
}

/// POST /api/send-location — kirim lokasi (appkey/authkey/to/lat/lng/name).
pub async fn send_location(State(state): State<AppState>, msg: ApiMsg) -> Response {
    let (sid, uid) = match auth_session(&state, &msg.appkey, &msg.authkey).await {
        Ok(v) => v,
        Err(e) => return e,
    };
    let to = msg.to.trim();
    let lat: f64 = msg.lat.trim().parse().unwrap_or(f64::NAN);
    let lng: f64 = msg.lng.trim().parse().unwrap_or(f64::NAN);
    if to.is_empty() || lat.is_nan() || lng.is_nan() {
        return err("validation", "'to', 'lat', 'lng' wajib diisi (angka)", StatusCode::UNPROCESSABLE_ENTITY);
    }
    let (_, max_msg, _, _) = quota::plan_limits(&state.pool, uid).await;
    if max_msg > 0 && quota::sent_today(&state.pool, uid).await >= max_msg {
        return err("quota_exceeded", "Kuota pesan harian habis.", StatusCode::FORBIDDEN);
    }
    if msg.sandbox.as_deref() == Some("true") {
        return (StatusCode::OK, Json(json!({"status":true,"message_status":"Success","message":"sandbox","id":sid.to_string()}))).into_response();
    }
    match state.wa.send_location(&sid.to_string(), to, lat, lng, msg.name.trim()).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status":true,"message_status":"Success","message":"queued","id":sid.to_string(),"data":{"to":to}}))).into_response(),
        Err(e) => err("gateway_error", &format!("gagal kirim ke gateway: {e}"), StatusCode::BAD_GATEWAY),
    }
}
