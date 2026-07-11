//! Klien HTTP ke sidecar WhatsApp (Baileys). Semua panggilan loopback + header X-WA-Secret.
//! Dipegang di `AppState.wa`.

use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct Gateway {
    base: String,
    secret: String,
    http: reqwest::Client,
}

#[derive(Deserialize, Default)]
pub struct QrResp {
    pub status: String,
    pub qr: Option<String>,
}

impl Gateway {
    pub fn from_env() -> Self {
        Self {
            base: std::env::var("WA_SIDECAR_URL").unwrap_or_else(|_| "http://127.0.0.1:8099".into()),
            secret: std::env::var("WA_SHARED_SECRET").unwrap_or_default(),
            http: reqwest::Client::new(),
        }
    }

    fn req(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        self.http
            .request(method, format!("{}{}", self.base, path))
            .header("X-WA-Secret", &self.secret)
    }

    pub async fn start(&self, session_id: &str, user_id: &str, level: &str, sim: bool) -> anyhow::Result<()> {
        self.req(reqwest::Method::POST, "/sessions/start")
            .json(&json!({ "session_id": session_id, "user_id": user_id, "level": level, "sim": sim }))
            .send().await?.error_for_status()?;
        Ok(())
    }

    /// Suntik pesan masuk SIMULASI (mode sim/dry) — untuk uji E2E tanpa nomor nyata.
    pub async fn sim_incoming(&self, session_id: &str, from: &str, text: &str) -> anyhow::Result<()> {
        self.req(reqwest::Method::POST, &format!("/sessions/{session_id}/sim/incoming"))
            .json(&json!({ "from": from, "text": text }))
            .send().await?.error_for_status()?;
        Ok(())
    }

    /// Kirim media (gambar/dokumen/video) via URL → masuk antrian throttle.
    pub async fn send_media(&self, session_id: &str, to: &str, media_url: &str, media_type: &str, caption: &str) -> anyhow::Result<Value> {
        Ok(self
            .req(reqwest::Method::POST, &format!("/sessions/{session_id}/send-media"))
            .json(&json!({ "to": to, "media_url": media_url, "type": media_type, "caption": caption }))
            .send().await?.error_for_status()?.json().await?)
    }

    /// Kirim lokasi → masuk antrian throttle.
    pub async fn send_location(&self, session_id: &str, to: &str, lat: f64, lng: f64, name: &str) -> anyhow::Result<Value> {
        Ok(self
            .req(reqwest::Method::POST, &format!("/sessions/{session_id}/send-location"))
            .json(&json!({ "to": to, "lat": lat, "lng": lng, "name": name }))
            .send().await?.error_for_status()?.json().await?)
    }

    /// Ubah level anti-ban sesi secara live (tanpa reconnect).
    pub async fn set_level(&self, session_id: &str, level: &str) -> anyhow::Result<()> {
        self.req(reqwest::Method::POST, &format!("/sessions/{session_id}/level"))
            .json(&json!({ "level": level }))
            .send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn qr(&self, session_id: &str) -> anyhow::Result<QrResp> {
        Ok(self
            .req(reqwest::Method::GET, &format!("/sessions/{session_id}/qr"))
            .send().await?.error_for_status()?.json().await?)
    }

    pub async fn send(&self, session_id: &str, to: &str, text: &str) -> anyhow::Result<Value> {
        Ok(self
            .req(reqwest::Method::POST, &format!("/sessions/{session_id}/send"))
            .json(&json!({ "to": to, "text": text }))
            .send().await?.error_for_status()?.json().await?)
    }

    pub async fn logout(&self, session_id: &str) -> anyhow::Result<()> {
        self.req(reqwest::Method::POST, &format!("/sessions/{session_id}/logout"))
            .send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn delete(&self, session_id: &str, purge: bool) -> anyhow::Result<()> {
        let q = if purge { "?purge=1" } else { "" };
        self.req(reqwest::Method::DELETE, &format!("/sessions/{session_id}{q}"))
            .send().await?.error_for_status()?;
        Ok(())
    }
}
