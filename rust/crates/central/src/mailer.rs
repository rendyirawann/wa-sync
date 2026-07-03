//! Pengiriman email via SMTP (lettre, async + native-tls/SChannel di Windows).
//! Konfigurasi dibaca dari env `MAIL_*`. Bila `MAIL_HOST` kosong → dianggap tidak
//! dikonfigurasi (caller fallback ke log link, cocok untuk dev).

use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

/// True bila SMTP dikonfigurasi (MAIL_HOST ada & tidak kosong).
pub fn configured() -> bool {
    std::env::var("MAIL_HOST").map(|h| !h.trim().is_empty()).unwrap_or(false)
}

fn env_or(key: &str, default: &str) -> String {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => v,
        _ => default.to_string(),
    }
}

/// Kirim email reset password. Mengembalikan true bila terkirim.
pub async fn send_reset(to: &str, reset_url: &str) -> bool {
    if !configured() {
        return false;
    }
    let host = env_or("MAIL_HOST", "");
    let port: u16 = std::env::var("MAIL_PORT").ok().and_then(|p| p.trim().parse().ok()).unwrap_or(465);
    let username = env_or("MAIL_USERNAME", "");
    let password = env_or("MAIL_PASSWORD", "");
    let from_addr = env_or("MAIL_FROM_ADDRESS", &username);
    let from_name = env_or("MAIL_FROM_NAME", "WA Service");
    let scheme = env_or("MAIL_SCHEME", "");

    let from_mbox = match format!("{from_name} <{from_addr}>").parse() {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("MAIL_FROM tidak valid: {e}");
            return false;
        }
    };
    let to_mbox = match to.parse() {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("alamat tujuan tidak valid ({to}): {e}");
            return false;
        }
    };

    let body = format!(
        "<div style=\"font-family:Arial,sans-serif;max-width:480px;margin:auto\">\
           <h2 style=\"color:#15a85b\">Reset Password</h2>\
           <p>Kami menerima permintaan reset password untuk akun WA Service Anda.</p>\
           <p style=\"margin:24px 0\"><a href=\"{reset_url}\" \
             style=\"background:#15a85b;color:#fff;padding:12px 22px;border-radius:8px;text-decoration:none;font-weight:bold\">\
             Reset Password</a></p>\
           <p style=\"color:#666;font-size:13px\">Atau salin tautan ini (berlaku 60 menit):<br>{reset_url}</p>\
           <p style=\"color:#999;font-size:12px\">Abaikan email ini jika Anda tidak meminta reset.</p>\
         </div>"
    );

    let email = match Message::builder()
        .from(from_mbox)
        .to(to_mbox)
        .subject("Reset Password — WA Service")
        .header(ContentType::TEXT_HTML)
        .body(body)
    {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("gagal menyusun email: {e}");
            return false;
        }
    };

    // smtps (port 465) → implicit TLS (relay); selain itu STARTTLS (587).
    let implicit_tls = scheme.eq_ignore_ascii_case("smtps") || port == 465;
    let builder = if implicit_tls {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
    };
    let transport = match builder {
        Ok(b) => b.port(port).credentials(Credentials::new(username, password)).build(),
        Err(e) => {
            tracing::error!("gagal inisialisasi SMTP: {e}");
            return false;
        }
    };

    match transport.send(email).await {
        Ok(_) => {
            tracing::info!("email reset terkirim ke {to}");
            true
        }
        Err(e) => {
            tracing::error!("gagal kirim email ke {to}: {e}");
            false
        }
    }
}
