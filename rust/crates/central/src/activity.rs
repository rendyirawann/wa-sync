use chrono::{Local, NaiveDateTime};
use serde_json::{json, Value};
use sqlx::types::Uuid;
use sqlx::PgPool;

/// Bangun objek "agent" (browser/os/device) dari User-Agent, meniru Jenssegers\Agent.
pub fn agent_json(ua: &str) -> Value {
    let parser = woothee::parser::Parser::new();
    if let Some(r) = parser.parse(ua) {
        let is_mobile = matches!(r.category, "smartphone" | "mobilephone");
        let is_desktop = r.category == "pc";
        json!({
            "browser": format!("{} {}", r.name, r.version).trim().to_string(),
            "os": r.os.to_string(),
            "device": r.category.to_string(),
            "is_mobile": is_mobile,
            "is_desktop": is_desktop,
            "raw": ua,
        })
    } else {
        json!({ "browser": "-", "os": "-", "device": "Unknown", "is_mobile": false, "is_desktop": false, "raw": ua })
    }
}

/// Tulis 1 baris ke `activity_log`. `extra` (objek) digabung ke properties bersama ip+agent.
pub async fn log(
    pool: &PgPool,
    log_name: &str,
    description: &str,
    causer_id: Uuid,
    ip: &str,
    ua: &str,
    extra: Value,
) {
    let mut props = json!({ "ip": ip, "agent": agent_json(ua) });
    if let (Some(obj), Some(ex)) = (props.as_object_mut(), extra.as_object()) {
        for (k, v) in ex {
            obj.insert(k.clone(), v.clone());
        }
    }
    let _ = sqlx::query(
        "INSERT INTO activity_log (log_name, event, description, causer_type, causer_id, properties, created_at, updated_at) \
         VALUES ($1, NULL, $2, $3, $4, $5::json, now(), now())",
    )
    .bind(log_name)
    .bind(description)
    .bind("App\\Models\\User")
    .bind(causer_id)
    .bind(props.to_string())
    .execute(pool)
    .await;

    // Broadcast ke admin yang online (WebSocket). Non-blocking; abai bila tak ada subscriber.
    crate::notify::event(log_name, description, &format!("IP {ip}"));
}

/// "X minutes ago" sederhana (meniru Carbon diffForHumans, bahasa Inggris).
pub fn human_diff(dt: NaiveDateTime) -> String {
    let now = Local::now().naive_local();
    let secs = (now - dt).num_seconds();
    if secs < 0 {
        return "just now".into();
    }
    let (n, unit) = if secs < 60 {
        (secs, "second")
    } else if secs < 3600 {
        (secs / 60, "minute")
    } else if secs < 86400 {
        (secs / 3600, "hour")
    } else if secs < 604800 {
        (secs / 86400, "day")
    } else if secs < 2_592_000 {
        (secs / 604800, "week")
    } else if secs < 31_536_000 {
        (secs / 2_592_000, "month")
    } else {
        (secs / 31_536_000, "year")
    };
    if n <= 0 {
        "just now".into()
    } else {
        format!("{n} {unit}{} ago", if n == 1 { "" } else { "s" })
    }
}

/// Format tanggal gaya Indonesia "d F Y, H:i" (mis. 12 Juni 2026, 00:06).
pub fn format_id_datetime(dt: NaiveDateTime) -> String {
    const BULAN: [&str; 12] = [
        "Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September",
        "Oktober", "November", "Desember",
    ];
    use chrono::{Datelike, Timelike};
    format!(
        "{:02} {} {}, {:02}:{:02}",
        dt.day(),
        BULAN[(dt.month() as usize).saturating_sub(1).min(11)],
        dt.year(),
        dt.hour(),
        dt.minute()
    )
}

/// Format "d F Y H:i:s" gaya Indonesia (mis. 12 Juni 2026 00:43:38).
pub fn format_id_full(dt: NaiveDateTime) -> String {
    const BULAN: [&str; 12] = [
        "Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September",
        "Oktober", "November", "Desember",
    ];
    use chrono::{Datelike, Timelike};
    format!(
        "{:02} {} {} {:02}:{:02}:{:02}",
        dt.day(),
        BULAN[(dt.month() as usize).saturating_sub(1).min(11)],
        dt.year(),
        dt.hour(),
        dt.minute(),
        dt.second()
    )
}

/// Ekstrak (ip, "os - browser", device_html) dari properties untuk tabel log.
pub fn agent_cols(properties: &Value) -> (String, String, String) {
    let ip = properties.get("ip").and_then(Value::as_str).unwrap_or("-").to_string();
    let agent = properties.get("agent");
    let browser = agent.and_then(|a| a.get("browser")).and_then(Value::as_str).unwrap_or("-");
    let os = agent.and_then(|a| a.get("os")).and_then(Value::as_str).unwrap_or("-");
    let is_desktop = agent.and_then(|a| a.get("is_desktop")).and_then(Value::as_bool).unwrap_or(false);
    let is_mobile = agent.and_then(|a| a.get("is_mobile")).and_then(Value::as_bool).unwrap_or(false);
    let device_raw = agent.and_then(|a| a.get("device")).and_then(Value::as_str).unwrap_or("Unknown");
    let device_html = if is_desktop {
        "<i class=\"ki-outline ki-screen text-primary me-2\"></i>Desktop".to_string()
    } else if is_mobile {
        "<i class=\"ki-outline ki-phone text-warning me-2\"></i>Mobile".to_string()
    } else {
        format!("<i class=\"ki-outline ki-question-2 text-danger me-2\"></i>{device_raw}")
    };
    (ip, format!("{os} - {browser}"), device_html)
}

/// Bangun 1 baris DataTable log (ip/device/os/description/created_at) dgn HTML — dipakai
/// di user-show-log, my-activity, my-login-session, dan log-activity.
pub fn dt_row(created_at: Option<NaiveDateTime>, description: Option<&str>, properties: &Value) -> Value {
    let ip = properties.get("ip").and_then(Value::as_str).unwrap_or("-");
    let agent = properties.get("agent");
    let browser = agent.and_then(|a| a.get("browser")).and_then(Value::as_str).unwrap_or("-");
    let os = agent.and_then(|a| a.get("os")).and_then(Value::as_str).unwrap_or("-");
    let is_desktop = agent.and_then(|a| a.get("is_desktop")).and_then(Value::as_bool).unwrap_or(false);
    let is_mobile = agent.and_then(|a| a.get("is_mobile")).and_then(Value::as_bool).unwrap_or(false);
    let device_raw = agent.and_then(|a| a.get("device")).and_then(Value::as_str).unwrap_or("Unknown");

    let created_html = match created_at {
        Some(dt) => format!(
            "<div class=\"text-end\"><label class=\"badge badge-info\">{}</label></div>",
            human_diff(dt)
        ),
        None => "<div class=\"text-end\"><label class=\"badge badge-warning\">Belum Pernah Login</label></div>".to_string(),
    };

    let device_html = if is_desktop {
        "<i class=\"ki-outline ki-screen text-primary me-2\"></i>Desktop".to_string()
    } else if is_mobile {
        "<i class=\"ki-outline ki-phone text-warning me-2\"></i>Mobile".to_string()
    } else {
        format!("<i class=\"ki-outline ki-question-2 text-danger me-2\"></i>{device_raw}")
    };

    json!({
        "ip": ip,
        "device": device_html,
        "os": format!("{os} - {browser}"),
        "description": description.unwrap_or("-"),
        "created_at": created_html,
    })
}
