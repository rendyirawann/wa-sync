//! Kontak / CRM (Batch 2): daftar kontak (dari pesan + yang disimpan), nama, TAG, catatan.
//! Bisa difilter per tag → dipakai untuk broadcast per-segmen. Scoped per-user (Superadmin semua).

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::types::Uuid;
use std::collections::{BTreeSet, HashMap};
use tower_sessions::Session;

use crate::{auth, rbac::CurrentUser, view, AppState};

const SELECT: &str = "SELECT x.session_id, x.jid, COALESCE(c.name,''), COALESCE(c.tags,''), COALESCE(c.notes,''), s.label \
    FROM (SELECT session_id, remote_jid AS jid FROM wa_messages UNION SELECT session_id, jid FROM wa_contacts) x \
    JOIN wa_sessions s ON s.id = x.session_id \
    LEFT JOIN wa_contacts c ON c.session_id = x.session_id AND c.jid = x.jid \
    WHERE ($1::uuid IS NULL OR s.user_id = $1) \
      AND ($2 = '' OR c.tags ILIKE '%' || $2 || '%') \
    ORDER BY s.label, x.jid";

pub async fn index(user: CurrentUser, session: Session, State(state): State<AppState>, Query(q): Query<HashMap<String, String>>) -> Html<String> {
    let scope: Option<Uuid> = if user.is_superadmin() { None } else { Some(user.id) };
    let tag = q.get("tag").cloned().unwrap_or_default();
    type Row = (Uuid, String, String, String, String, String);
    let rows: Vec<Row> = sqlx::query_as(SELECT)
        .bind(scope)
        .bind(&tag)
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    let contacts: Vec<_> = rows
        .iter()
        .map(|r| {
            let phone = r.1.split('@').next().unwrap_or(&r.1).to_string();
            let tags: Vec<String> = r.3.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect();
            json!({
                "session_id": r.0.to_string(), "jid": r.1, "phone": phone,
                "name": r.2, "tags": tags, "tags_raw": r.3, "notes": r.4, "session_label": r.5,
            })
        })
        .collect();

    // Kumpulan tag unik (untuk filter chips) — dihitung dari semua kontak user (tanpa filter tag).
    let all: Vec<(String,)> = sqlx::query_as(
        "SELECT COALESCE(c.tags,'') FROM wa_contacts c JOIN wa_sessions s ON s.id=c.session_id \
         WHERE ($1::uuid IS NULL OR s.user_id=$1) AND c.tags <> ''",
    )
    .bind(scope)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();
    let mut tagset: BTreeSet<String> = BTreeSet::new();
    for (t,) in &all {
        for one in t.split(',') {
            let x = one.trim();
            if !x.is_empty() {
                tagset.insert(x.to_string());
            }
        }
    }
    let all_tags: Vec<String> = tagset.into_iter().collect();

    let mut ctx = view::base_context(&state, &session, &user, "contacts").await;
    ctx.insert("contacts", &contacts);
    ctx.insert("all_tags", &all_tags);
    ctx.insert("filter_tag", &tag);
    match state.tera.render("wa/contacts.html", &ctx) {
        Ok(h) => Html(h),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

async fn owns_session(state: &AppState, user: &CurrentUser, sid: Uuid) -> bool {
    let found: Option<(Uuid,)> = if user.is_superadmin() {
        sqlx::query_as("SELECT id FROM wa_sessions WHERE id=$1").bind(sid).fetch_optional(&state.pool).await.ok().flatten()
    } else {
        sqlx::query_as("SELECT id FROM wa_sessions WHERE id=$1 AND user_id=$2").bind(sid).bind(user.id).fetch_optional(&state.pool).await.ok().flatten()
    };
    found.is_some()
}

#[derive(Deserialize)]
pub struct SaveForm {
    session_id: String,
    jid: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    tags: String,
    #[serde(default)]
    notes: String,
    #[serde(rename = "_token", default)]
    token: String,
}

/// Simpan nama/tag/catatan kontak (persona AI tak disentuh).
pub async fn save(user: CurrentUser, session: Session, State(state): State<AppState>, Form(f): Form<SaveForm>) -> Response {
    if !auth::verify_csrf(&session, &f.token).await {
        return (StatusCode::from_u16(419).unwrap(), "Sesi kedaluwarsa").into_response();
    }
    let Ok(sid) = Uuid::parse_str(&f.session_id) else {
        return (StatusCode::BAD_REQUEST, "bad session").into_response();
    };
    if !owns_session(&state, &user, sid).await {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    // Normalisasi tag → "a, b, c"
    let tags: Vec<String> = f.tags.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect();
    let tags = tags.join(", ");
    let _ = sqlx::query(
        "INSERT INTO wa_contacts (session_id, jid, name, tags, notes, updated_at) VALUES ($1,$2,$3,$4,$5,now()) \
         ON CONFLICT (session_id, jid) DO UPDATE SET name=EXCLUDED.name, tags=EXCLUDED.tags, notes=EXCLUDED.notes, updated_at=now()",
    )
    .bind(sid).bind(f.jid.trim()).bind(f.name.trim()).bind(&tags).bind(f.notes.trim())
    .execute(&state.pool).await;
    view::set_flash(&session, "success", "Kontak disimpan.").await;
    Redirect::to("/admin/wa/contacts").into_response()
}
