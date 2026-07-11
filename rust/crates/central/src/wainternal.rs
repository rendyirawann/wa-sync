//! Endpoint internal yang dipanggil SIDECAR: `POST /internal/wa/event`.
//! Tidak di belakang require_auth — diamankan dgn header X-WA-Secret (constant-time)
//! + cek loopback. Menyimpan event ke DB lalu push ke browser via notify (WS).

use axum::{
    extract::{ConnectInfo, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::types::Uuid;
use std::net::SocketAddr;
use subtle::ConstantTimeEq;

use crate::{notify, AppState};

#[derive(Deserialize)]
pub struct WaEvent {
    pub session_id: String,
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(default)]
    pub data: Value,
}

fn secret_ok(headers: &HeaderMap) -> bool {
    let want = std::env::var("WA_SHARED_SECRET").unwrap_or_default();
    if want.is_empty() {
        return false;
    }
    let got = headers.get("x-wa-secret").and_then(|v| v.to_str().ok()).unwrap_or("");
    got.as_bytes().ct_eq(want.as_bytes()).into()
}

pub async fn event(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(ev): Json<WaEvent>,
) -> impl IntoResponse {
    if !addr.ip().is_loopback() {
        return (StatusCode::FORBIDDEN, "forbidden");
    }
    if !secret_ok(&headers) {
        return (StatusCode::UNAUTHORIZED, "unauthorized");
    }
    let Ok(sid) = Uuid::parse_str(&ev.session_id) else {
        return (StatusCode::BAD_REQUEST, "bad session_id");
    };
    let pool = &state.pool;

    match ev.typ.as_str() {
        "qr" => {
            let qr = ev.data.get("qr").and_then(Value::as_str);
            let _ = sqlx::query("UPDATE wa_sessions SET status='qr', qr=$1, updated_at=now() WHERE id=$2")
                .bind(qr).bind(sid).execute(pool).await;
        }
        "connected" => {
            let phone = ev.data.get("phone").and_then(Value::as_str);
            let jid = ev.data.get("jid").and_then(Value::as_str);
            let _ = sqlx::query("UPDATE wa_sessions SET status='connected', qr=NULL, phone=$1, jid=$2, last_connected_at=now(), updated_at=now() WHERE id=$3")
                .bind(phone).bind(jid).bind(sid).execute(pool).await;
        }
        "disconnected" => {
            let _ = sqlx::query("UPDATE wa_sessions SET status='disconnected', updated_at=now() WHERE id=$1")
                .bind(sid).execute(pool).await;
        }
        "banned" => {
            let _ = sqlx::query("UPDATE wa_sessions SET status='banned', updated_at=now() WHERE id=$1")
                .bind(sid).execute(pool).await;
        }
        "logged_out" => {
            let _ = sqlx::query("UPDATE wa_sessions SET status='logged_out', qr=NULL, updated_at=now() WHERE id=$1")
                .bind(sid).execute(pool).await;
        }
        "message" => {
            let remote = ev.data.get("remote_jid").and_then(Value::as_str).unwrap_or("");
            let body = ev.data.get("body").and_then(Value::as_str);
            let waid = ev.data.get("wa_msg_id").and_then(Value::as_str);
            let mtype = ev.data.get("msg_type").and_then(Value::as_str).unwrap_or("text");
            let murl = ev.data.get("media_url").and_then(Value::as_str);
            let mmime = ev.data.get("media_mime").and_then(Value::as_str);
            let mname = ev.data.get("media_name").and_then(Value::as_str);
            let _ = sqlx::query(
                "INSERT INTO wa_messages (id, session_id, direction, remote_jid, body, msg_type, media_url, media_mime, media_name, wa_message_id, status) \
                 VALUES ($1,$2,'in',$3,$4,$5,$6,$7,$8,$9,'received') \
                 ON CONFLICT (session_id, wa_message_id) WHERE wa_message_id IS NOT NULL DO NOTHING",
            )
            .bind(Uuid::now_v7()).bind(sid).bind(remote).bind(body).bind(mtype).bind(murl).bind(mmime).bind(mname).bind(waid)
            .execute(pool).await;
            crate::quota::bump(pool, sid, "received").await;
            // Balas otomatis (non-blocking): keyword rule dulu, lalu AI. Guardrail di dalam.
            let incoming = body.unwrap_or("").to_string();
            tokio::spawn(crate::autoreply::handle(state.clone(), sid, remote.to_string(), incoming));
        }
        "sent" => {
            // hasil kirim keluar dari antrian throttle sidecar
            let remote = ev.data.get("remote_jid").and_then(Value::as_str).unwrap_or("");
            let body = ev.data.get("body").and_then(Value::as_str);
            let waid = ev.data.get("wa_msg_id").and_then(Value::as_str);
            let st = ev.data.get("status").and_then(Value::as_str).unwrap_or("sent");
            let err = ev.data.get("error").and_then(Value::as_str);
            let _ = sqlx::query(
                "INSERT INTO wa_messages (id, session_id, direction, remote_jid, body, msg_type, wa_message_id, status, error) \
                 VALUES ($1,$2,'out',$3,$4,'text',$5,$6,$7) \
                 ON CONFLICT (session_id, wa_message_id) WHERE wa_message_id IS NOT NULL DO NOTHING",
            )
            .bind(Uuid::now_v7()).bind(sid).bind(remote).bind(body).bind(waid).bind(st).bind(err)
            .execute(pool).await;
            crate::quota::bump(pool, sid, if st == "failed" { "failed" } else { "sent" }).await;
        }
        "status" => {
            // Update centang pesan keluar (sent→delivered→read), tak pernah turun peringkat.
            let waid = ev.data.get("wa_msg_id").and_then(Value::as_str).unwrap_or("");
            let status = ev.data.get("status").and_then(Value::as_str).unwrap_or("");
            if !waid.is_empty() && !status.is_empty() {
                let _ = sqlx::query(
                    "UPDATE wa_messages SET status=$3 WHERE session_id=$1 AND wa_message_id=$2 AND direction='out' \
                     AND (CASE status WHEN 'played' THEN 5 WHEN 'read' THEN 4 WHEN 'delivered' THEN 3 WHEN 'sent' THEN 2 ELSE 1 END) \
                       < (CASE $3 WHEN 'played' THEN 5 WHEN 'read' THEN 4 WHEN 'delivered' THEN 3 WHEN 'sent' THEN 2 ELSE 1 END)",
                )
                .bind(sid).bind(waid).bind(status).execute(pool).await;
            }
        }
        _ => {}
    }

    // cari pemilik sesi → push event HANYA ke browser pemilik (+ Superadmin lihat semua)
    let owner: Option<(Uuid,)> = sqlx::query_as("SELECT user_id FROM wa_sessions WHERE id=$1")
        .bind(sid).fetch_optional(pool).await.ok().flatten();
    let to_user = owner.map(|o| o.0.to_string()).unwrap_or_default();
    notify::wa(&to_user, &ev.session_id, &ev.typ, ev.data.clone());
    // Webhook keluar (non-blocking; difilter per config sesi).
    crate::webhook::dispatch(state.pool.clone(), sid, &ev.typ, ev.data.clone());
    (StatusCode::OK, "ok")
}

/// GET /internal/wa/usage/{id} — dipanggil sidecar saat start untuk MENGEMBALIKAN counter cap
/// dari DB (jumlah pesan keluar hari ini & 1 jam terakhir), agar cap anti-ban PERSISTEN
/// melintasi restart sidecar (tidak reset ke 0). Dijaga secret + loopback seperti /event.
pub async fn usage(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if !addr.ip().is_loopback() {
        return (StatusCode::FORBIDDEN, "forbidden").into_response();
    }
    if !secret_ok(&headers) {
        return (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    }
    let Ok(sid) = Uuid::parse_str(&id) else {
        return (StatusCode::BAD_REQUEST, "bad session_id").into_response();
    };
    let (day, hour): (i64, i64) = sqlx::query_as(
        "SELECT \
            (SELECT count(*) FROM wa_messages WHERE session_id=$1 AND direction='out' AND created_at::date = CURRENT_DATE), \
            (SELECT count(*) FROM wa_messages WHERE session_id=$1 AND direction='out' AND created_at >= now() - interval '1 hour')",
    )
    .bind(sid)
    .fetch_one(&state.pool)
    .await
    .unwrap_or((0, 0));
    Json(json!({ "day_sent": day, "hour_sent": hour })).into_response()
}
