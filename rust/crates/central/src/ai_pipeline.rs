//! Pipeline AI auto-reply: dipanggil (spawn) saat pesan masuk. Cek config + guardrail,
//! bangun konteks dari riwayat, panggil Ollama, kirim balasan via antrian throttle.

use sqlx::types::Uuid;

use crate::{ai, quota, AppState};

pub async fn maybe_reply(state: AppState, session_id: Uuid, remote_jid: String) {
    if remote_jid.ends_with("@g.us") {
        return; // grup tidak dibalas (jaga-jaga)
    }
    let cfg: Option<(bool, String, String, i32, Uuid, i16, i16, i32)> = sqlx::query_as(
        "SELECT ai_enabled, ai_model, ai_system_prompt, ai_history_limit, user_id, ai_hours_start, ai_hours_end, ai_only_when_idle_min \
         FROM wa_sessions WHERE id = $1 AND status = 'connected'",
    )
    .bind(session_id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    let Some((enabled, model, sys, hist, uid, hstart, hend, idle_min)) = cfg else { return };
    if !enabled {
        return;
    }
    // Human handoff: kalau AI dijeda untuk kontak ini (mis. sedang ditangani agen), lewati.
    let paused: Option<bool> = sqlx::query_scalar("SELECT ai_paused FROM wa_contacts WHERE session_id=$1 AND jid=$2")
        .bind(session_id).bind(&remote_jid).fetch_optional(&state.pool).await.ok().flatten();
    if paused == Some(true) {
        return;
    }
    // "Hanya saat idle": kalau agen (manusia) baru saja membalas (< N menit), jangan ganggu.
    if idle_min > 0 {
        let recent: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM wa_messages WHERE session_id=$1 AND remote_jid=$2 AND direction='out' \
             AND via_ai=false AND created_at > now() - make_interval(mins => $3::int) LIMIT 1",
        )
        .bind(session_id).bind(&remote_jid).bind(idle_min)
        .fetch_optional(&state.pool).await.ok().flatten();
        if recent.is_some() {
            return;
        }
    }
    // AI hanya untuk plan yang mengizinkan (Premium/Enterprise) — Superadmin (pemilik platform) bypass.
    let is_super: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM roles r JOIN model_has_roles mhr ON mhr.role_id=r.id \
         WHERE mhr.model_id=$1 AND lower(r.name)='superadmin')",
    )
    .bind(uid)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(false);
    if !is_super {
        let (_, _, plan_ai, _) = quota::plan_limits(&state.pool, uid).await;
        if !plan_ai {
            return;
        }
    }
    // Jam kerja (server TZ Asia/Jakarta). start==end ⇒ 24 jam.
    if hstart != hend {
        use chrono::Timelike;
        let h = chrono::Local::now().hour() as i16;
        let within = if hstart < hend { h >= hstart && h < hend } else { h >= hstart || h < hend };
        if !within {
            return;
        }
    }
    // Riwayat percakapan (lama→baru) sebagai konteks.
    let rows: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT direction, body FROM wa_messages WHERE session_id = $1 AND remote_jid = $2 \
         ORDER BY created_at DESC LIMIT $3",
    )
    .bind(session_id)
    .bind(&remote_jid)
    .bind(hist.max(1) as i64)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let history: Vec<(String, String)> = rows
        .into_iter()
        .rev()
        .filter_map(|(d, b)| b.map(|body| ((if d == "in" { "user" } else { "assistant" }).to_string(), body)))
        .collect();
    if history.is_empty() {
        return;
    }
    // Persona AI KHUSUS per-kontak (mis. nomor pacar → balas mesra) menimpa prompt sesi.
    let persona: Option<String> = sqlx::query_scalar(
        "SELECT ai_persona FROM wa_contacts WHERE session_id=$1 AND jid=$2 AND ai_persona <> ''",
    )
    .bind(session_id)
    .bind(&remote_jid)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    let system = match persona {
        Some(p) if !p.trim().is_empty() => p,
        _ if !sys.trim().is_empty() => sys,
        _ => ai::DEFAULT_SYSTEM.to_string(),
    };
    if let Some(reply) = ai::reply(&model, &system, &history).await {
        // Kirim balik ke JID ASLI (lengkap dgn domain) supaya balasan sampai ke chat yg benar,
        // termasuk alamat @lid baru WhatsApp. Sidecar memakai JID apa adanya bila mengandung '@'.
        if state.wa.send(&session_id.to_string(), &remote_jid, &reply).await.is_ok() {
            quota::bump(&state.pool, session_id, "ai_reply").await;
        }
    }
}
