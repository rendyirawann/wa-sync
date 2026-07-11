//! Knowledge base AI (Batch 4, RAG-lite): FAQ / katalog per-nomor. Saat AI membalas,
//! entri yang relevan (skor overlap kata dengan pesan user) disuntik ke system prompt.
//! Retrieval sederhana berbasis kata kunci (tanpa embedding) — cukup untuk FAQ/katalog.

use std::collections::HashSet;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Form, Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::types::Uuid;
use sqlx::PgPool;
use tower_sessions::Session;

use crate::{auth, rbac::CurrentUser, view, AppState};

/// Ambil konteks pengetahuan relevan untuk `query` → blok teks untuk system prompt.
pub async fn retrieve(pool: &PgPool, session_id: Uuid, query: &str) -> Option<String> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT title, content FROM wa_knowledge WHERE session_id=$1 AND enabled AND content <> '' ORDER BY created_at",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    if rows.is_empty() {
        return None;
    }
    let qtokens: HashSet<String> = query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() > 2)
        .map(|t| t.to_string())
        .collect();
    let mut scored: Vec<(i32, &(String, String))> = rows
        .iter()
        .map(|r| {
            let text = format!("{} {}", r.0, r.1).to_lowercase();
            let score = qtokens.iter().filter(|t| text.contains(t.as_str())).count() as i32;
            (score, r)
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));

    let mut out = String::from("Basis pengetahuan (pakai bila relevan dengan pertanyaan):\n");
    let mut budget: usize = 3000;
    let mut used = 0;
    for (score, (title, content)) in scored {
        if score == 0 && used >= 2 {
            break; // sertakan beberapa umum, tapi jangan semua kalau tak match
        }
        let entry = if title.trim().is_empty() {
            format!("- {}\n", content.trim())
        } else {
            format!("- {}: {}\n", title.trim(), content.trim())
        };
        if entry.len() > budget {
            continue;
        }
        out.push_str(&entry);
        budget -= entry.len();
        used += 1;
        if used >= 6 {
            break;
        }
    }
    if used == 0 {
        return None;
    }
    out.push_str("Jawab berdasarkan info di atas bila relevan. Jika informasinya tidak ada, jujur katakan dan tawarkan untuk menyambungkan ke admin.");
    Some(out)
}

// ===================== Halaman kelola =====================

pub async fn index(user: CurrentUser, session: Session, State(state): State<AppState>) -> Html<String> {
    type S = (Uuid, String, Option<String>);
    let sessions: Vec<S> = if user.is_superadmin() {
        sqlx::query_as("SELECT id, label, phone FROM wa_sessions ORDER BY label").fetch_all(&state.pool).await.unwrap_or_default()
    } else {
        sqlx::query_as("SELECT id, label, phone FROM wa_sessions WHERE user_id=$1 ORDER BY label").bind(user.id).fetch_all(&state.pool).await.unwrap_or_default()
    };
    let sess: Vec<_> = sessions.iter().map(|s| json!({ "id": s.0.to_string(), "label": s.1, "phone": s.2.clone().unwrap_or_default() })).collect();

    type R = (Uuid, String, String, bool, String);
    let rows: Vec<R> = if user.is_superadmin() {
        sqlx::query_as("SELECT k.id, k.title, k.content, k.enabled, COALESCE(s.label,'') FROM wa_knowledge k JOIN wa_sessions s ON s.id=k.session_id ORDER BY s.label, k.created_at")
            .fetch_all(&state.pool).await.unwrap_or_default()
    } else {
        sqlx::query_as("SELECT k.id, k.title, k.content, k.enabled, COALESCE(s.label,'') FROM wa_knowledge k JOIN wa_sessions s ON s.id=k.session_id WHERE s.user_id=$1 ORDER BY s.label, k.created_at")
            .bind(user.id).fetch_all(&state.pool).await.unwrap_or_default()
    };
    let items: Vec<_> = rows.iter().map(|r| json!({
        "id": r.0.to_string(), "title": r.1, "content": r.2, "enabled": r.3, "session_label": r.4
    })).collect();

    let mut ctx = view::base_context(&state, &session, &user, "knowledge").await;
    ctx.insert("sessions", &sess);
    ctx.insert("items", &items);
    match state.tera.render("wa/knowledge.html", &ctx) {
        Ok(h) => Html(h),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

async fn owns_session(state: &AppState, user: &CurrentUser, sid: Uuid) -> bool {
    let f: Option<(Uuid,)> = if user.is_superadmin() {
        sqlx::query_as("SELECT id FROM wa_sessions WHERE id=$1").bind(sid).fetch_optional(&state.pool).await.ok().flatten()
    } else {
        sqlx::query_as("SELECT id FROM wa_sessions WHERE id=$1 AND user_id=$2").bind(sid).bind(user.id).fetch_optional(&state.pool).await.ok().flatten()
    };
    f.is_some()
}

#[derive(Deserialize)]
pub struct KbForm {
    #[serde(default)]
    session_id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    content: String,
    #[serde(rename = "_token", default)]
    token: String,
}

pub async fn create(user: CurrentUser, session: Session, State(state): State<AppState>, Form(f): Form<KbForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    let Ok(sid) = Uuid::parse_str(f.session_id.trim()) else {
        view::set_flash(&session, "error", "Pilih nomor.").await;
        return Redirect::to("/admin/wa/knowledge").into_response();
    };
    if !owns_session(&state, &user, sid).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    if f.content.trim().is_empty() {
        view::set_flash(&session, "error", "Isi pengetahuan tidak boleh kosong.").await;
        return Redirect::to("/admin/wa/knowledge").into_response();
    }
    let _ = sqlx::query("INSERT INTO wa_knowledge (id, session_id, title, content) VALUES ($1,$2,$3,$4)")
        .bind(Uuid::now_v7()).bind(sid).bind(f.title.trim()).bind(f.content.trim()).execute(&state.pool).await;
    view::set_flash(&session, "success", "Pengetahuan ditambahkan.").await;
    Redirect::to("/admin/wa/knowledge").into_response()
}

pub async fn destroy(user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    let res = if user.is_superadmin() {
        sqlx::query("DELETE FROM wa_knowledge WHERE id=$1").bind(id).execute(&state.pool).await
    } else {
        sqlx::query("DELETE FROM wa_knowledge k USING wa_sessions s WHERE k.id=$1 AND k.session_id=s.id AND s.user_id=$2")
            .bind(id).bind(user.id).execute(&state.pool).await
    };
    match res {
        Ok(_) => Json(json!({ "status": "ok" })).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response(),
    }
}
