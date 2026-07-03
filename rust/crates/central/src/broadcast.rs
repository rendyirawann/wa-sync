//! Broadcast / Campaign (Fase C). Kirim pesan massal ke banyak kontak lewat 1 nomor,
//! dengan template + variabel `{{nama}}` + spintax `{a|b}`, kontrol kecepatan (memakai
//! antrian throttle anti-ban sidecar yang sudah ada), penjadwalan, dan laporan.
//!
//! Pelacakan status target = OPSI A (optimistik): "sent" berarti pesan BERHASIL MASUK
//! ANTRIAN throttle sidecar (lalu dikirim bertahap), bukan konfirmasi sampai ke HP.

use std::collections::HashSet;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Form, Json,
};
use chrono::{FixedOffset, NaiveDateTime, TimeZone};
use serde::Deserialize;
use serde_json::json;
use sqlx::types::Uuid;
use tower_sessions::Session;

use crate::{auth, rbac::CurrentUser, view, AppState};

/// Batas target per campaign (lindungi RAM antrian sidecar + UX). Sisa di-skip + diberi tahu.
const MAX_TARGETS: usize = 5000;

// ===================== Halaman daftar =====================

pub async fn index(user: CurrentUser, session: Session, State(state): State<AppState>) -> Html<String> {
    type C = (Uuid, String, String, String, Option<NaiveDateTime>, Option<NaiveDateTime>, i32, i32, i32);
    let camps: Vec<C> = if user.is_superadmin() {
        sqlx::query_as(
            "SELECT c.id, c.name, c.status, COALESCE(s.label,'—'), c.scheduled_at::timestamp, c.created_at::timestamp, c.total, c.sent_count, c.failed_count \
             FROM wa_campaigns c LEFT JOIN wa_sessions s ON s.id=c.session_id ORDER BY c.created_at DESC",
        )
        .fetch_all(&state.pool).await.unwrap_or_default()
    } else {
        sqlx::query_as(
            "SELECT c.id, c.name, c.status, COALESCE(s.label,'—'), c.scheduled_at::timestamp, c.created_at::timestamp, c.total, c.sent_count, c.failed_count \
             FROM wa_campaigns c LEFT JOIN wa_sessions s ON s.id=c.session_id WHERE c.user_id=$1 ORDER BY c.created_at DESC",
        )
        .bind(user.id).fetch_all(&state.pool).await.unwrap_or_default()
    };
    let campaigns: Vec<_> = camps
        .iter()
        .map(|c| {
            let done = c.7 + c.8;
            let pct = if c.6 > 0 { (done as f64 / c.6 as f64 * 100.0).round() as i64 } else { 0 };
            json!({
                "id": c.0.to_string(), "name": c.1, "status": c.2, "session": c.3,
                "scheduled": c.4.map(fmt_dt).unwrap_or_default(),
                "created": c.5.map(fmt_dt).unwrap_or_default(),
                "total": c.6, "sent": c.7, "failed": c.8, "pct": pct,
            })
        })
        .collect();

    // sesi connected (untuk pilih nomor pengirim)
    type S = (Uuid, String, Option<String>);
    let sess: Vec<S> = if user.is_superadmin() {
        sqlx::query_as("SELECT id, label, phone FROM wa_sessions WHERE status='connected' ORDER BY label")
            .fetch_all(&state.pool).await.unwrap_or_default()
    } else {
        sqlx::query_as("SELECT id, label, phone FROM wa_sessions WHERE status='connected' AND user_id=$1 ORDER BY label")
            .bind(user.id).fetch_all(&state.pool).await.unwrap_or_default()
    };
    let sessions: Vec<_> = sess.iter().map(|s| json!({
        "id": s.0.to_string(), "label": s.1, "phone": s.2.clone().unwrap_or_default()
    })).collect();

    // template tersimpan
    type T = (Uuid, String, String);
    let tpls: Vec<T> = sqlx::query_as("SELECT id, name, body FROM wa_message_templates WHERE user_id=$1 ORDER BY created_at DESC")
        .bind(user.id).fetch_all(&state.pool).await.unwrap_or_default();
    let templates: Vec<_> = tpls.iter().map(|t| json!({
        "id": t.0.to_string(), "name": t.1, "body": t.2
    })).collect();

    let mut ctx = view::base_context(&state, &session, &user, "broadcast").await;
    ctx.insert("campaigns", &campaigns);
    ctx.insert("sessions", &sessions);
    ctx.insert("templates", &templates);
    match state.tera.render("wa/broadcast.html", &ctx) {
        Ok(h) => Html(h),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

fn fmt_dt(d: NaiveDateTime) -> String {
    d.format("%d %b %Y, %H:%M").to_string()
}

// ===================== Buat campaign =====================

#[derive(Deserialize)]
pub struct CreateCampaign {
    #[serde(default)]
    session_id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    targets: String,
    /// Bila diisi: ambil penerima dari Kontak dengan tag ini (mengabaikan daftar manual).
    #[serde(default)]
    segment_tag: String,
    #[serde(default)]
    schedule_at: String,
    #[serde(rename = "_token", default)]
    token: String,
}

pub async fn create(user: CurrentUser, session: Session, State(state): State<AppState>, Form(f): Form<CreateCampaign>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    let redir = Redirect::to("/admin/wa/broadcast").into_response();

    // Validasi sesi pengirim (milik user / Superadmin).
    let Ok(sid) = Uuid::parse_str(f.session_id.trim()) else {
        view::set_flash(&session, "error", "Pilih nomor pengirim terlebih dahulu.").await;
        return redir;
    };
    if !owns_session(&state, &user, sid).await {
        view::set_flash(&session, "error", "Nomor pengirim tidak valid.").await;
        return redir;
    }
    let body = f.body.trim();
    if body.is_empty() {
        view::set_flash(&session, "error", "Isi pesan tidak boleh kosong.").await;
        return redir;
    }
    // Sumber penerima: SEGMEN (tag kontak) bila diisi, else daftar manual.
    let (parsed, dropped) = if !f.segment_tag.trim().is_empty() {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT jid, COALESCE(name,'') FROM wa_contacts WHERE session_id=$1 AND tags ILIKE '%'||$2||'%'",
        )
        .bind(sid).bind(f.segment_tag.trim())
        .fetch_all(&state.pool).await.unwrap_or_default();
        let list: Vec<(String, Option<String>)> = rows
            .iter()
            .filter_map(|(jid, name)| {
                let ph: String = jid.split('@').next().unwrap_or(jid).chars().filter(|c| c.is_ascii_digit()).collect();
                if ph.len() >= 8 {
                    Some((ph, if name.trim().is_empty() { None } else { Some(name.trim().to_string()) }))
                } else {
                    None
                }
            })
            .collect();
        (list, 0usize)
    } else {
        parse_targets(&f.targets, MAX_TARGETS)
    };
    if parsed.is_empty() {
        view::set_flash(&session, "error", "Daftar penerima kosong. Isi daftar manual atau pilih segmen tag yang ada kontaknya.").await;
        return redir;
    }
    let name = if f.name.trim().is_empty() { "Broadcast".to_string() } else { f.name.trim().to_string() };
    let schedule = parse_schedule(&f.schedule_at);

    // Insert campaign (draft dulu).
    let cid = Uuid::now_v7();
    if sqlx::query("INSERT INTO wa_campaigns (id, user_id, session_id, name, body, status) VALUES ($1,$2,$3,$4,$5,'draft')")
        .bind(cid).bind(user.id).bind(sid).bind(&name).bind(body)
        .execute(&state.pool).await.is_err()
    {
        view::set_flash(&session, "error", "Gagal membuat campaign.").await;
        return redir;
    }
    // Insert target (dedup nomor via unique index).
    for (phone, pname) in &parsed {
        let vars = json!({ "nama": pname.clone().unwrap_or_default() }).to_string();
        let _ = sqlx::query(
            "INSERT INTO wa_campaign_targets (id, campaign_id, phone, name, vars) VALUES ($1,$2,$3,$4,$5::jsonb) \
             ON CONFLICT (campaign_id, phone) DO NOTHING",
        )
        .bind(Uuid::now_v7()).bind(cid).bind(phone).bind(pname).bind(&vars)
        .execute(&state.pool).await;
    }
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM wa_campaign_targets WHERE campaign_id=$1")
        .bind(cid).fetch_one(&state.pool).await.unwrap_or(0);
    let _ = sqlx::query("UPDATE wa_campaigns SET total=$2 WHERE id=$1").bind(cid).bind(total as i32).execute(&state.pool).await;

    let mut note = format!("{total} penerima siap.");
    if dropped > 0 {
        note.push_str(&format!(" {dropped} baris dilewati (nomor tidak valid / melebihi batas {MAX_TARGETS})."));
    }

    if let Some(when) = schedule {
        let _ = sqlx::query("UPDATE wa_campaigns SET status='scheduled', scheduled_at=$2, updated_at=now() WHERE id=$1")
            .bind(cid).bind(when).execute(&state.pool).await;
        view::set_flash(&session, "success", &format!("Campaign dijadwalkan {}. {note}", fmt_dt(when.naive_local()))).await;
    } else {
        let _ = sqlx::query("UPDATE wa_campaigns SET status='running', started_at=now(), updated_at=now() WHERE id=$1")
            .bind(cid).execute(&state.pool).await;
        tokio::spawn(run_campaign(state.clone(), cid));
        view::set_flash(&session, "success", &format!("Campaign dimulai (pesan masuk antrian throttle). {note}")).await;
    }
    Redirect::to(&format!("/admin/wa/broadcast/{cid}")).into_response()
}

// ===================== Detail + laporan =====================

pub async fn show(user: CurrentUser, session: Session, State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    if !owns_campaign(&state, &user, id).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    type C = (String, String, String, Option<NaiveDateTime>, Option<NaiveDateTime>, i32, i32, i32, String, Option<String>);
    let Some(c): Option<C> = sqlx::query_as(
        "SELECT c.name, c.body, c.status, c.scheduled_at::timestamp, c.created_at::timestamp, c.total, c.sent_count, c.failed_count, COALESCE(s.label,'—'), s.phone \
         FROM wa_campaigns c LEFT JOIN wa_sessions s ON s.id=c.session_id WHERE c.id=$1",
    )
    .bind(id).fetch_optional(&state.pool).await.ok().flatten() else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };
    let pending = (c.5 - c.6 - c.7).max(0);
    let done = c.6 + c.7;
    let pct = if c.5 > 0 { (done as f64 / c.5 as f64 * 100.0).round() as i64 } else { 0 };

    type Tg = (String, Option<String>, String, Option<String>, Option<NaiveDateTime>);
    let tgts: Vec<Tg> = sqlx::query_as(
        "SELECT phone, name, status, error, sent_at::timestamp FROM wa_campaign_targets WHERE campaign_id=$1 \
         ORDER BY CASE status WHEN 'failed' THEN 0 WHEN 'pending' THEN 1 ELSE 2 END, id LIMIT 1000",
    )
    .bind(id).fetch_all(&state.pool).await.unwrap_or_default();
    let targets: Vec<_> = tgts.iter().map(|t| json!({
        "phone": t.0, "name": t.1.clone().unwrap_or_default(), "status": t.2,
        "error": t.3.clone().unwrap_or_default(),
        "sent_at": t.4.map(|d| d.format("%H:%M:%S").to_string()).unwrap_or_default(),
    })).collect();

    let mut ctx = view::base_context(&state, &session, &user, "broadcast").await;
    ctx.insert("c_id", &id.to_string());
    ctx.insert("c_name", &c.0);
    ctx.insert("c_body", &c.1);
    ctx.insert("c_status", &c.2);
    ctx.insert("c_scheduled", &c.3.map(fmt_dt).unwrap_or_default());
    ctx.insert("c_created", &c.4.map(fmt_dt).unwrap_or_default());
    ctx.insert("c_total", &c.5);
    ctx.insert("c_sent", &c.6);
    ctx.insert("c_failed", &c.7);
    ctx.insert("c_pending", &pending);
    ctx.insert("c_pct", &pct);
    ctx.insert("c_session", &c.8);
    ctx.insert("c_phone", &c.9.clone().unwrap_or_default());
    ctx.insert("targets", &targets);
    match state.tera.render("wa/broadcast_show.html", &ctx) {
        Ok(h) => Html(h).into_response(),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")).into_response(),
    }
}

/// JSON ringkas untuk polling progress di halaman detail.
pub async fn status(user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    if !owns_campaign(&state, &user, id).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let row: Option<(String, i32, i32, i32)> = sqlx::query_as(
        "SELECT status, total, sent_count, failed_count FROM wa_campaigns WHERE id=$1",
    )
    .bind(id).fetch_optional(&state.pool).await.ok().flatten();
    match row {
        Some((st, total, sent, failed)) => {
            let pending = (total - sent - failed).max(0);
            let pct = if total > 0 { ((sent + failed) as f64 / total as f64 * 100.0).round() as i64 } else { 0 };
            Json(json!({ "status": st, "total": total, "sent": sent, "failed": failed, "pending": pending, "pct": pct })).into_response()
        }
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

// ===================== Aksi: jalankan / batalkan =====================

#[derive(Deserialize)]
pub struct TokenForm {
    #[serde(rename = "_token", default)]
    token: String,
}

pub async fn run(user: CurrentUser, session: Session, State(state): State<AppState>, Path(id): Path<Uuid>, Form(f): Form<TokenForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    if !owns_campaign(&state, &user, id).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let updated = sqlx::query(
        "UPDATE wa_campaigns SET status='running', started_at=COALESCE(started_at,now()), scheduled_at=NULL, last_error=NULL, updated_at=now() \
         WHERE id=$1 AND status IN ('draft','scheduled','paused')",
    )
    .bind(id).execute(&state.pool).await.map(|r| r.rows_affected()).unwrap_or(0);
    if updated > 0 {
        tokio::spawn(run_campaign(state.clone(), id));
        view::set_flash(&session, "success", "Campaign dijalankan.").await;
    } else {
        view::set_flash(&session, "info", "Campaign tidak dalam status yang bisa dijalankan.").await;
    }
    Redirect::to(&format!("/admin/wa/broadcast/{id}")).into_response()
}

pub async fn cancel(user: CurrentUser, session: Session, State(state): State<AppState>, Path(id): Path<Uuid>, Form(f): Form<TokenForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    if !owns_campaign(&state, &user, id).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let _ = sqlx::query(
        "UPDATE wa_campaigns SET status='canceled', finished_at=now(), updated_at=now() \
         WHERE id=$1 AND status IN ('draft','scheduled','running','paused')",
    )
    .bind(id).execute(&state.pool).await;
    view::set_flash(&session, "info", "Campaign dibatalkan. Pesan yang sudah masuk antrian tetap terkirim.").await;
    Redirect::to(&format!("/admin/wa/broadcast/{id}")).into_response()
}

// ===================== Template pesan (reusable) =====================

#[derive(Deserialize)]
pub struct TemplateForm {
    #[serde(default)]
    name: String,
    #[serde(default)]
    body: String,
    #[serde(rename = "_token", default)]
    token: String,
}

pub async fn tpl_store(user: CurrentUser, session: Session, State(state): State<AppState>, Form(f): Form<TemplateForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    let name = f.name.trim();
    let body = f.body.trim();
    if name.is_empty() || body.is_empty() {
        view::set_flash(&session, "error", "Nama & isi template wajib diisi.").await;
        return Redirect::to("/admin/wa/broadcast").into_response();
    }
    let _ = sqlx::query("INSERT INTO wa_message_templates (id, user_id, name, body) VALUES ($1,$2,$3,$4)")
        .bind(Uuid::now_v7()).bind(user.id).bind(name).bind(body).execute(&state.pool).await;
    view::set_flash(&session, "success", "Template disimpan.").await;
    Redirect::to("/admin/wa/broadcast").into_response()
}

pub async fn tpl_destroy(user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    let res = if user.is_superadmin() {
        sqlx::query("DELETE FROM wa_message_templates WHERE id=$1").bind(id).execute(&state.pool).await
    } else {
        sqlx::query("DELETE FROM wa_message_templates WHERE id=$1 AND user_id=$2").bind(id).bind(user.id).execute(&state.pool).await
    };
    match res {
        Ok(_) => Json(json!({ "status": "ok" })).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response(),
    }
}

// ===================== Worker & scheduler =====================

/// Eksekusi campaign: kirim tiap target lewat antrian throttle sidecar (Opsi A optimistik).
pub async fn run_campaign(state: AppState, campaign_id: Uuid) {
    let pool = &state.pool;
    let camp: Option<(Uuid, Uuid, String)> = sqlx::query_as(
        "SELECT user_id, session_id, body FROM wa_campaigns WHERE id=$1 AND status IN ('running','scheduled','paused')",
    )
    .bind(campaign_id).fetch_optional(pool).await.ok().flatten();
    let Some((user_id, session_id, body)) = camp else { return };
    let _ = sqlx::query("UPDATE wa_campaigns SET status='running', started_at=COALESCE(started_at,now()), updated_at=now() WHERE id=$1 AND status<>'canceled'")
        .bind(campaign_id).execute(pool).await;

    // Superadmin (atau cap<=0) = tanpa batas harian.
    let is_super: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM roles r JOIN model_has_roles mhr ON mhr.role_id=r.id WHERE mhr.model_id=$1 AND lower(r.name)='superadmin')",
    )
    .bind(user_id).fetch_one(pool).await.unwrap_or(false);
    let (_, max_msgs, _, _) = crate::quota::plan_limits(pool, user_id).await;
    let base_sent = crate::quota::sent_today(pool, user_id).await;
    let mut enqueued: i64 = 0;

    loop {
        // dibatalkan?
        let st: Option<String> = sqlx::query_scalar("SELECT status FROM wa_campaigns WHERE id=$1")
            .bind(campaign_id).fetch_optional(pool).await.ok().flatten();
        if st.as_deref() != Some("running") {
            return;
        }
        // batas kuota harian paket
        if !is_super && max_msgs > 0 && base_sent + enqueued >= max_msgs {
            let _ = sqlx::query("UPDATE wa_campaigns SET status='paused', last_error='Kuota pesan harian paket tercapai — lanjutkan besok / upgrade plan.', updated_at=now() WHERE id=$1 AND status='running'")
                .bind(campaign_id).execute(pool).await;
            return;
        }
        // klaim 1 target pending secara atomik (anti dobel-kirim antar worker)
        let claimed: Option<(Uuid, String, String)> = sqlx::query_as(
            "UPDATE wa_campaign_targets SET status='sent', sent_at=now() \
             WHERE id = (SELECT id FROM wa_campaign_targets WHERE campaign_id=$1 AND status='pending' ORDER BY id FOR UPDATE SKIP LOCKED LIMIT 1) \
             RETURNING id, phone, vars::text",
        )
        .bind(campaign_id).fetch_optional(pool).await.ok().flatten();
        let Some((tid, phone, vars_text)) = claimed else { break };

        let vmap = serde_json::from_str::<serde_json::Value>(&vars_text)
            .ok()
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();
        let text = crate::broadcast_engine::render(&body, &vmap, &[&campaign_id.to_string(), &phone]);

        match state.wa.send(&session_id.to_string(), &phone, &text).await {
            Ok(resp) => {
                let waid = resp
                    .get("id")
                    .or_else(|| resp.get("wa_msg_id"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let _ = sqlx::query("UPDATE wa_campaign_targets SET rendered_body=$2, wa_message_id=$3 WHERE id=$1")
                    .bind(tid).bind(&text).bind(&waid).execute(pool).await;
                let _ = sqlx::query("UPDATE wa_campaigns SET sent_count=sent_count+1, updated_at=now() WHERE id=$1")
                    .bind(campaign_id).execute(pool).await;
                enqueued += 1;
            }
            Err(e) => {
                let msg = e.to_string();
                let _ = sqlx::query("UPDATE wa_campaign_targets SET status='failed', rendered_body=$2, error=$3, sent_at=NULL WHERE id=$1")
                    .bind(tid).bind(&text).bind(&msg).execute(pool).await;
                let _ = sqlx::query("UPDATE wa_campaigns SET failed_count=failed_count+1, last_error=$2, updated_at=now() WHERE id=$1")
                    .bind(campaign_id).bind(&msg).execute(pool).await;
            }
        }
    }

    // selesai bila tak ada lagi pending
    let _ = sqlx::query(
        "UPDATE wa_campaigns SET status='done', finished_at=now(), updated_at=now() \
         WHERE id=$1 AND status='running' AND NOT EXISTS (SELECT 1 FROM wa_campaign_targets WHERE campaign_id=$1 AND status='pending')",
    )
    .bind(campaign_id).execute(pool).await;
}

/// Tick penjadwal (dipanggil periodik dari main.rs): klaim campaign 'scheduled' yang due → jalankan.
pub async fn scheduler_tick(state: &AppState) {
    let due: Vec<(Uuid,)> = sqlx::query_as(
        "UPDATE wa_campaigns SET status='running', started_at=COALESCE(started_at,now()), updated_at=now() \
         WHERE status='scheduled' AND COALESCE(scheduled_at, now()) <= now() RETURNING id",
    )
    .fetch_all(&state.pool).await.unwrap_or_default();
    for (id,) in due {
        tokio::spawn(run_campaign(state.clone(), id));
    }
}

/// Saat boot: lanjutkan campaign yang berstatus 'running' (mis. server sempat restart).
pub async fn resume_running(state: &AppState) {
    let ids: Vec<(Uuid,)> = sqlx::query_as("SELECT id FROM wa_campaigns WHERE status='running'")
        .fetch_all(&state.pool).await.unwrap_or_default();
    for (id,) in ids {
        tokio::spawn(run_campaign(state.clone(), id));
    }
}

// ===================== Helper =====================

async fn owns_session(state: &AppState, user: &CurrentUser, id: Uuid) -> bool {
    let found: Option<(Uuid,)> = if user.is_superadmin() {
        sqlx::query_as("SELECT id FROM wa_sessions WHERE id=$1").bind(id).fetch_optional(&state.pool).await.ok().flatten()
    } else {
        sqlx::query_as("SELECT id FROM wa_sessions WHERE id=$1 AND user_id=$2").bind(id).bind(user.id).fetch_optional(&state.pool).await.ok().flatten()
    };
    found.is_some()
}

async fn owns_campaign(state: &AppState, user: &CurrentUser, id: Uuid) -> bool {
    let found: Option<(Uuid,)> = if user.is_superadmin() {
        sqlx::query_as("SELECT id FROM wa_campaigns WHERE id=$1").bind(id).fetch_optional(&state.pool).await.ok().flatten()
    } else {
        sqlx::query_as("SELECT id FROM wa_campaigns WHERE id=$1 AND user_id=$2").bind(id).bind(user.id).fetch_optional(&state.pool).await.ok().flatten()
    };
    found.is_some()
}

/// Normalisasi nomor Indonesia ke format 62xxxx (buang non-digit, 0→62, 8→62).
fn normalize_phone(raw: &str) -> Option<String> {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    let d = if let Some(rest) = digits.strip_prefix('0') {
        format!("62{rest}")
    } else if digits.starts_with("62") {
        digits
    } else if digits.starts_with('8') {
        format!("62{digits}")
    } else {
        digits
    };
    if (10..=15).contains(&d.len()) { Some(d) } else { None }
}

/// Parse textarea daftar penerima: "628xxx,Nama" per baris. Dedup nomor, batasi `cap`.
/// Mengembalikan (target valid, jumlah baris dilewati).
fn parse_targets(raw: &str, cap: usize) -> (Vec<(String, Option<String>)>, usize) {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<(String, Option<String>)> = Vec::new();
    let mut dropped = 0usize;
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(2, |c| c == ',' || c == ';' || c == '\t');
        let phone_raw = parts.next().unwrap_or("").trim();
        let name = parts.next().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        match normalize_phone(phone_raw) {
            Some(p) => {
                if seen.contains(&p) {
                    continue; // duplikat → diam-diam dilewati
                }
                if out.len() >= cap {
                    dropped += 1;
                    continue;
                }
                seen.insert(p.clone());
                out.push((p, name));
            }
            None => dropped += 1,
        }
    }
    (out, dropped)
}

/// Parse input datetime-local ("2026-06-13T15:30") sebagai waktu WIB (+07:00). Kosong → None.
fn parse_schedule(s: &str) -> Option<chrono::DateTime<FixedOffset>> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let nd = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M")
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S"))
        .ok()?;
    let wib = FixedOffset::east_opt(7 * 3600)?;
    wib.from_local_datetime(&nd).single()
}
