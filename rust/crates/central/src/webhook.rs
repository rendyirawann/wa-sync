//! Webhook keluar per-nomor: saat ada event WA, POST ke `webhook_url` subscriber.
//! Ditandatangani HMAC-SHA256 (header `X-WA-Signature: sha256=<hex>`), retry + log.

use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha256;
use sqlx::types::Uuid;
use sqlx::PgPool;

fn sign(secret: &str, body: &str) -> String {
    let mut mac = match Hmac::<Sha256>::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return String::new(),
    };
    mac.update(body.as_bytes());
    mac.finalize().into_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

/// Non-blocking: cek config sesi → kirim webhook bila aktif & event diminta. Retry [0,2,6]s.
pub fn dispatch(pool: PgPool, session_id: Uuid, event: &str, payload: Value) {
    let event = event.to_string();
    tokio::spawn(async move {
        let row: Option<(Option<String>, Option<String>, bool, Vec<String>)> = sqlx::query_as(
            "SELECT webhook_url, webhook_secret, webhook_enabled, webhook_events FROM wa_sessions WHERE id=$1",
        )
        .bind(session_id)
        .fetch_optional(&pool)
        .await
        .ok()
        .flatten();
        let Some((Some(url), secret, enabled, events)) = row else { return };
        if !enabled || url.trim().is_empty() || !events.iter().any(|e| e == &event) {
            return;
        }
        let body = json!({ "event": event, "session_id": session_id.to_string(), "data": payload }).to_string();
        let sig = sign(secret.as_deref().unwrap_or(""), &body);
        let client = reqwest::Client::new();
        let mut delivered = false;
        let mut code: Option<i32> = None;
        let mut last_err: Option<String> = None;
        let mut attempts = 0;
        for delay in [0u64, 2, 6] {
            if delay > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
            }
            attempts += 1;
            let res = client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("X-WA-Signature", format!("sha256={sig}"))
                .timeout(std::time::Duration::from_secs(10))
                .body(body.clone())
                .send()
                .await;
            match res {
                Ok(resp) => {
                    code = Some(resp.status().as_u16() as i32);
                    if resp.status().is_success() {
                        delivered = true;
                        break;
                    }
                    last_err = Some(format!("HTTP {}", resp.status()));
                }
                Err(e) => last_err = Some(e.to_string()),
            }
        }
        let _ = sqlx::query(
            "INSERT INTO wa_webhook_deliveries (id,session_id,event,url,status_code,attempts,last_error,delivered) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(Uuid::now_v7()).bind(session_id).bind(&event).bind(&url)
        .bind(code).bind(attempts).bind(last_err).bind(delivered)
        .execute(&pool).await;
    });
}
