//! Klien Midtrans Snap + verifikasi signature. Murni (tanpa DB) agar mudah diuji.
//! Kredensial dibaca dari tabel `settings` (lihat billing.rs / settings.rs).

use std::collections::HashMap;

use serde_json::json;

/// Konfigurasi Midtrans dari `settings` (key midtrans_*).
#[derive(Clone, Debug)]
pub struct MidtransCfg {
    pub enabled: bool,
    pub is_production: bool,
    pub server_key: String,
    pub client_key: String,
}

impl MidtransCfg {
    /// Baca konfig: Settings DB dulu, jatuh ke variabel .env bila kosong.
    pub fn from_settings(s: &HashMap<String, String>) -> Self {
        let g = |k: &str| s.get(k).cloned().unwrap_or_default();
        // DB setting → fallback env
        let pick = |dbk: &str, envk: &str| {
            let v = g(dbk);
            if v.trim().is_empty() { std::env::var(envk).unwrap_or_default() } else { v }
        };
        let server_key = pick("midtrans_server_key", "MIDTRANS_SERVER_KEY");
        let client_key = pick("midtrans_client_key", "MIDTRANS_CLIENT_KEY");
        // production: setting DB non-kosong menang; else env (true/1)
        let db_prod = g("midtrans_is_production");
        let is_production = if !db_prod.trim().is_empty() {
            db_prod == "1"
        } else {
            matches!(std::env::var("MIDTRANS_IS_PRODUCTION").unwrap_or_default().trim(), "true" | "1")
        };
        // aktif bila toggle DB "1" ATAU server key tersedia (dari .env)
        let enabled = g("midtrans_enabled") == "1" || !server_key.trim().is_empty();
        Self { enabled, is_production, server_key, client_key }
    }

    /// Siap dipakai bila diaktifkan + server key terisi.
    pub fn is_configured(&self) -> bool {
        self.enabled && !self.server_key.trim().is_empty()
    }

    /// Base URL Snap (transaksi) sesuai mode.
    fn snap_base(&self) -> &'static str {
        if self.is_production {
            "https://app.midtrans.com"
        } else {
            "https://app.sandbox.midtrans.com"
        }
    }
}

/// Buat transaksi Snap → (snap_token, redirect_url).
/// `amount` = rupiah bulat (gross_amount Midtrans = integer untuk IDR).
pub async fn create_snap_transaction(
    cfg: &MidtransCfg,
    order_id: &str,
    amount: i64,
    item_id: &str,
    item_name: &str,
    customer_name: &str,
    customer_email: &str,
    finish_url: &str,
) -> anyhow::Result<(String, String)> {
    let body = json!({
        "transaction_details": { "order_id": order_id, "gross_amount": amount },
        "item_details": [{ "id": item_id, "price": amount, "quantity": 1, "name": item_name }],
        "customer_details": { "first_name": customer_name, "email": customer_email },
        "callbacks": { "finish": finish_url }
    });
    let url = format!("{}/snap/v1/transactions", cfg.snap_base());
    let mut req = reqwest::Client::new()
        .post(&url)
        .basic_auth(&cfg.server_key, Some(""))
        .header("Accept", "application/json");
    // Override URL notifikasi per-transaksi bila MIDTRANS_NOTIFY_URL diisi (else pakai dashboard).
    let notify = std::env::var("MIDTRANS_NOTIFY_URL").unwrap_or_default();
    if !notify.trim().is_empty() {
        req = req.header("X-Override-Notification", notify.trim());
    }
    let resp = req
        .json(&body)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?;
    let status = resp.status();
    let v: serde_json::Value = resp.json().await?;
    if !status.is_success() {
        let msg = v.get("error_messages").map(|e| e.to_string()).unwrap_or_else(|| v.to_string());
        anyhow::bail!("Midtrans {status}: {msg}");
    }
    let token = v.get("token").and_then(|t| t.as_str()).unwrap_or_default().to_string();
    let redirect = v.get("redirect_url").and_then(|t| t.as_str()).unwrap_or_default().to_string();
    if token.is_empty() || redirect.is_empty() {
        anyhow::bail!("Midtrans: respon tanpa token/redirect_url ({v})");
    }
    Ok((token, redirect))
}

/// Verifikasi signature notifikasi: SHA512(order_id + status_code + gross_amount + server_key),
/// hex lowercase, dibandingkan constant-time.
pub fn verify_signature(
    server_key: &str,
    order_id: &str,
    status_code: &str,
    gross_amount: &str,
    got_sig: &str,
) -> bool {
    use sha2::{Digest, Sha512};
    use subtle::ConstantTimeEq;
    if server_key.is_empty() || got_sig.is_empty() {
        return false;
    }
    let mut h = Sha512::new();
    h.update(order_id.as_bytes());
    h.update(status_code.as_bytes());
    h.update(gross_amount.as_bytes());
    h.update(server_key.as_bytes());
    let want: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    want.as_bytes().ct_eq(got_sig.trim().to_lowercase().as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_roundtrip() {
        // Hitung signature yang benar lalu pastikan verify menerimanya, dan menolak yang salah.
        use sha2::{Digest, Sha512};
        let (sk, oid, sc, amt) = ("Mid-server-xyz", "WA-123", "200", "99000.00");
        let mut h = Sha512::new();
        h.update(oid.as_bytes());
        h.update(sc.as_bytes());
        h.update(amt.as_bytes());
        h.update(sk.as_bytes());
        let sig: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
        assert!(verify_signature(sk, oid, sc, amt, &sig));
        assert!(verify_signature(sk, oid, sc, amt, &sig.to_uppercase())); // case-insensitive
        assert!(!verify_signature(sk, oid, sc, amt, "deadbeef"));
        assert!(!verify_signature(sk, oid, sc, amt, ""));
        assert!(!verify_signature("", oid, sc, amt, &sig)); // server key kosong → tolak
    }
}
