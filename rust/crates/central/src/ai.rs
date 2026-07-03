//! Klien Ollama (chat) untuk AI auto-reply. Default model qwen2.5 (Bahasa Indonesia bagus).
//! Bila Ollama mati / error → kembalikan None (caller diam, tidak menggantung).

use serde_json::json;
use std::time::Duration;
use tokio::sync::Semaphore;

pub const DEFAULT_SYSTEM: &str = "Kamu adalah asisten customer service toko via WhatsApp. \
Jawab dalam Bahasa Indonesia yang ramah, singkat (1-3 kalimat), dan membantu. \
Bila tidak tahu atau butuh tindakan admin, katakan akan diteruskan ke admin. \
Jangan mengarang harga, stok, atau janji yang tidak pasti.";

// Batasi inferensi paralel (model 7B lokal berat) agar tidak membebani CPU/GPU.
static SEM: Semaphore = Semaphore::const_new(2);

/// history: urut kronologis (lama→baru), role "user"/"assistant".
pub async fn reply(model: &str, system: &str, history: &[(String, String)]) -> Option<String> {
    let _permit = SEM.acquire().await.ok()?;
    let base = std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".into());
    let mut messages = vec![json!({ "role": "system", "content": system })];
    for (role, content) in history {
        messages.push(json!({ "role": role, "content": content }));
    }
    let body = json!({ "model": model, "messages": messages, "stream": false, "options": { "temperature": 0.6 } });
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base}/api/chat"))
        .timeout(Duration::from_secs(60))
        .json(&body)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        tracing::warn!("Ollama HTTP {}", resp.status());
        return None;
    }
    let v: serde_json::Value = resp.json().await.ok()?;
    v.get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
