//! Notifikasi realtime via WebSocket.
//!
//! Channel broadcast global (1 → banyak). Setiap browser admin membuka koneksi WS
//! ke `/admin/ws/notifications`; server mem-broadcast event ke semua koneksi.
//! Sumber event sekarang: `activity::log` (tiap aktivitas tercatat) + endpoint
//! `POST /admin/notify`. Di Fase B, gateway Baileys memanggil `notify::event(...)`
//! untuk push event WA (pesan masuk, sesi connect, dll) secara langsung.

use std::sync::OnceLock;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
    Json,
};
use serde_json::{json, Value};
use tokio::sync::broadcast;

use crate::rbac::CurrentUser;

static NOTIFIER: OnceLock<broadcast::Sender<String>> = OnceLock::new();

/// Inisialisasi channel broadcast (panggil sekali di `main`).
pub fn init() {
    let (tx, _rx) = broadcast::channel::<String>(256);
    let _ = NOTIFIER.set(tx);
}

fn sender() -> Option<&'static broadcast::Sender<String>> {
    NOTIFIER.get()
}

/// Kirim payload JSON mentah ke semua klien WS yang terhubung.
pub fn push(payload: String) {
    if let Some(tx) = sender() {
        let _ = tx.send(payload); // Err bila belum ada subscriber — abaikan.
    }
}

/// Bentuk lalu broadcast satu notifikasi terstruktur.
pub fn event(kind: &str, title: &str, body: &str) {
    let ts = chrono::Local::now().format("%H:%M").to_string();
    push(json!({ "kind": kind, "title": title, "body": body, "ts": ts }).to_string());
}

/// Broadcast event WhatsApp (kind="wa") ke browser admin — channel sama dgn bell.
/// Browser membedakan via `kind` lalu mengarahkan ke handler halaman sesi.
pub fn wa(to_user: &str, session_id: &str, event: &str, data: Value) {
    push(json!({ "kind": "wa", "to_user": to_user, "session_id": session_id, "event": event, "data": data }).to_string());
}

/// `GET /admin/ws/notifications` — upgrade ke WebSocket (di belakang require_auth).
pub async fn ws(ws: WebSocketUpgrade, user: CurrentUser) -> Response {
    let name = user.name.clone();
    let my_id = user.id.to_string();
    let is_super = user.is_superadmin();
    ws.on_upgrade(move |socket| handle(socket, name, my_id, is_super))
}

/// Filter pesan broadcast per-koneksi (multi-tenant): Superadmin terima semua;
/// selain itu hanya event WA miliknya (`to_user`) + sapaan `system`.
fn should_send(payload: &str, my_id: &str, is_super: bool) -> bool {
    if is_super {
        return true;
    }
    match serde_json::from_str::<Value>(payload) {
        Ok(v) => match v.get("kind").and_then(|k| k.as_str()) {
            Some("wa") => v.get("to_user").and_then(|t| t.as_str()) == Some(my_id),
            Some("system") => true,
            _ => false, // notifikasi aktivitas platform → hanya Superadmin
        },
        Err(_) => false,
    }
}

async fn handle(mut socket: WebSocket, name: String, my_id: String, is_super: bool) {
    // Sapaan awal — dijamin sampai ke klien yang baru terhubung.
    let hello = json!({
        "kind": "system",
        "title": "Notifikasi realtime aktif",
        "body": format!("Halo {name}, update akan muncul di sini secara langsung."),
        "ts": chrono::Local::now().format("%H:%M").to_string(),
    })
    .to_string();
    if socket.send(Message::Text(hello.into())).await.is_err() {
        return;
    }

    let mut rx = match sender() {
        Some(tx) => tx.subscribe(),
        None => return,
    };

    loop {
        tokio::select! {
            msg = rx.recv() => match msg {
                Ok(payload) => {
                    if should_send(&payload, &my_id, is_super)
                        && socket.send(Message::Text(payload.into())).await.is_err()
                    {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            },
            client = socket.recv() => match client {
                Some(Ok(Message::Close(_))) | None => break,
                Some(Ok(_)) => {}            // ping/teks dari klien — abaikan
                Some(Err(_)) => break,
            },
        }
    }
}

/// Body untuk `POST /admin/notify` (pemicu manual / hook Fase B).
#[derive(serde::Deserialize)]
pub struct NotifyIn {
    #[serde(default)]
    pub kind: String,
    pub title: String,
    #[serde(default)]
    pub body: String,
}

/// `POST /admin/notify` — kirim notifikasi ad-hoc ke semua admin yang online.
pub async fn trigger(_user: CurrentUser, Json(p): Json<NotifyIn>) -> Json<Value> {
    let kind = if p.kind.is_empty() { "info" } else { &p.kind };
    event(kind, &p.title, &p.body);
    Json(json!({ "status": "ok" }))
}
