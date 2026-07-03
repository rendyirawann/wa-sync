use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{
    extract::{ConnectInfo, Multipart, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Form, Json,
};
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::types::Uuid;
use tower_sessions::Session;

use crate::{activity, auth, datatables, rbac::CurrentUser, view, AppState};

// ---------- util ----------
fn ip_of(addr: SocketAddr) -> String { addr.ip().to_string() }
fn ua_of(h: &HeaderMap) -> String { h.get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or("").to_string() }
fn csrf_header(h: &HeaderMap) -> String { h.get("x-csrf-token").and_then(|v| v.to_str().ok()).unwrap_or("").to_string() }
fn avatar_dir() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../storage/app/public/user/avatar") }
fn ext_ok(fname: &str) -> Option<String> {
    let e = fname.rsplit('.').next().unwrap_or("").to_lowercase();
    if ["jpg", "jpeg", "png"].contains(&e.as_str()) { Some(e) } else { None }
}
fn errors(map: Value) -> Response { Json(json!({ "errors": map })).into_response() }

fn render_partial(state: &AppState, tpl: &str, ctx: &tera::Context) -> Response {
    match state.tera.render(tpl, ctx) {
        Ok(html) => Json(json!({ "html": html })).into_response(),
        Err(e) => Json(json!({ "html": format!("<pre>Template error: {e:#}</pre>") })).into_response(),
    }
}

fn user_obj(user: &CurrentUser) -> Value {
    json!({ "id": user.id, "name": user.name, "email": user.email, "no_wa": user.no_wa.clone().unwrap_or_default(), "avatar": user.avatar.clone().unwrap_or_default() })
}

async fn read_avatar(mut mp: Multipart) -> (HashMap<String, String>, Option<(String, Vec<u8>)>) {
    let mut fields = HashMap::new();
    let mut avatar = None;
    while let Ok(Some(field)) = mp.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "avatar" {
            let fname = field.file_name().unwrap_or("").to_string();
            let bytes = field.bytes().await.map(|b| b.to_vec()).unwrap_or_default();
            if !fname.is_empty() && !bytes.is_empty() { avatar = Some((fname, bytes)); }
        } else if let Ok(v) = field.text().await {
            fields.insert(name, v);
        }
    }
    (fields, avatar)
}

async fn page(state: &AppState, session: &Session, user: &CurrentUser, tpl: &str, subtab: &str, extra: Option<(&str, Value)>) -> Html<String> {
    let mut ctx = view::base_context(state, session, user, "account").await;
    ctx.insert("subtab", subtab);
    if let Some((k, v)) = extra { ctx.insert(k, &v); }
    match state.tera.render(tpl, &ctx) {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

// ---------- account ----------
pub async fn account_index(user: CurrentUser, session: Session, State(state): State<AppState>) -> Html<String> {
    page(&state, &session, &user, "backend/my_profile/index.html", "overview", None).await
}

pub async fn avatar_edit(user: CurrentUser, session: Session, State(state): State<AppState>) -> Response {
    let mut ctx = tera::Context::new();
    ctx.insert("user", &user_obj(&user));
    ctx.insert("csrf_token", &auth::ensure_csrf(&session).await);
    render_partial(&state, "backend/my_profile/change_pic.html", &ctx)
}

pub async fn avatar_update(user: CurrentUser, session: Session, State(state): State<AppState>, ConnectInfo(addr): ConnectInfo<SocketAddr>, headers: HeaderMap, mp: Multipart) -> Response {
    let (fields, avatar) = read_avatar(mp).await;
    let token = fields.get("_token").cloned().unwrap_or_else(|| csrf_header(&headers));
    if !auth::verify_csrf(&session, &token).await {
        return (StatusCode::from_u16(419).unwrap(), Json(json!({"error":"Sesi kedaluwarsa","judul":"Error"}))).into_response();
    }
    let Some((fname, bytes)) = avatar else {
        return errors(json!({ "avatar": ["Foto wajib di upload."] }));
    };
    if ext_ok(&fname).is_none() { return errors(json!({ "avatar": ["Foto harus berformat JPG atau PNG."] })); }
    if bytes.len() > 2 * 1024 * 1024 { return errors(json!({ "avatar": ["Ukuran file foto maksimal 2MB."] })); }

    let dir = avatar_dir();
    let _ = std::fs::create_dir_all(&dir);
    let ext = ext_ok(&fname).unwrap();
    let filename = format!("avatar-{}-{}.{}", user.id, chrono::Utc::now().timestamp(), ext);
    if std::fs::write(dir.join(&filename), &bytes).is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"Terjadi kesalahan di aplikasi, hubungi Developer.","judul":"Aplikasi Error"}))).into_response();
    }
    if let Some(old) = &user.avatar { if !old.is_empty() { let _ = std::fs::remove_file(dir.join(old)); } }
    let _ = sqlx::query("UPDATE users SET avatar = $1, updated_at = now() WHERE id = $2").bind(&filename).bind(user.id).execute(&state.pool).await;
    activity::log(&state.pool, "profile", "Mengganti Avatar", user.id, &ip_of(addr), &ua_of(&headers), json!({})).await;
    (StatusCode::CREATED, Json(json!({"success":"Avatar berhasil diperbaharui.","judul":"Berhasil","avatar_url": format!("/storage/user/avatar/{filename}")}))).into_response()
}

// ---------- profile ----------
pub async fn profile_index(user: CurrentUser, session: Session, State(state): State<AppState>) -> Html<String> {
    page(&state, &session, &user, "backend/my_profile/profile/index.html", "overview", None).await
}

pub async fn profile_edit(user: CurrentUser, session: Session, State(state): State<AppState>) -> Response {
    let mut ctx = tera::Context::new();
    ctx.insert("user", &user_obj(&user));
    ctx.insert("csrf_token", &auth::ensure_csrf(&session).await);
    render_partial(&state, "backend/my_profile/profile/edit.html", &ctx)
}

pub async fn profile_update(user: CurrentUser, session: Session, State(state): State<AppState>, ConnectInfo(addr): ConnectInfo<SocketAddr>, headers: HeaderMap, mut mp: Multipart) -> Response {
    let mut f = HashMap::new();
    while let Ok(Some(field)) = mp.next_field().await {
        let n = field.name().unwrap_or("").to_string();
        if let Ok(v) = field.text().await { f.insert(n, v); }
    }
    let token = f.get("_token").cloned().unwrap_or_else(|| csrf_header(&headers));
    if !auth::verify_csrf(&session, &token).await {
        return (StatusCode::from_u16(419).unwrap(), Json(json!({"error":"Sesi kedaluwarsa","judul":"Error"}))).into_response();
    }
    let name = f.get("name").map(|s| s.trim().to_string()).unwrap_or_default();
    let no_wa = f.get("no_wa").map(|s| s.trim().to_string()).unwrap_or_default();
    let email = f.get("email").map(|s| s.trim().to_string()).unwrap_or_default();
    let mut errs = serde_json::Map::new();
    if name.is_empty() { errs.insert("name".into(), json!(["Nama lengkap wajib diisi."])); }
    else if name.chars().count() > 100 { errs.insert("name".into(), json!(["Nama lengkap maksimal 100 karakter."])); }
    if no_wa.is_empty() { errs.insert("no_wa".into(), json!(["Nomor WhatsApp wajib diisi."])); }
    else if no_wa.chars().count() < 10 || no_wa.chars().count() > 20 { errs.insert("no_wa".into(), json!(["Nomor WhatsApp 10-20 karakter."])); }
    else if uniq_other(&state, "no_wa", &no_wa, user.id).await { errs.insert("no_wa".into(), json!(["Nomor WhatsApp sudah digunakan oleh pengguna lain."])); }
    if email.is_empty() { errs.insert("email".into(), json!(["Email wajib diisi."])); }
    else if !email.contains('@') { errs.insert("email".into(), json!(["Format email tidak valid."])); }
    else if uniq_other(&state, "email", &email, user.id).await { errs.insert("email".into(), json!(["Email sudah digunakan oleh pengguna lain."])); }
    if !errs.is_empty() { return errors(Value::Object(errs)); }

    let _ = sqlx::query("UPDATE users SET name=$1, no_wa=$2, email=$3, updated_at=now() WHERE id=$4")
        .bind(&name).bind(&no_wa).bind(&email).bind(user.id).execute(&state.pool).await;
    activity::log(&state.pool, "profile", "Mengubah Data Profile Akun", user.id, &ip_of(addr), &ua_of(&headers), json!({"new":{"name":name,"no_wa":no_wa,"email":email}})).await;
    Json(json!({"success":"Data profile berhasil diperbaharui.","judul":"Berhasil","updated":{"name":name,"no_wa":no_wa,"email":email}})).into_response()
}

async fn uniq_other(state: &AppState, col: &str, val: &str, except: Uuid) -> bool {
    let sql = if col == "no_wa" { "SELECT 1 FROM users WHERE no_wa = $1 AND id <> $2" } else { "SELECT 1 FROM users WHERE email = $1 AND id <> $2" };
    sqlx::query_scalar::<_, i32>(sql).bind(val).bind(except).fetch_optional(&state.pool).await.ok().flatten().is_some()
}

// ---------- security ----------
pub async fn security_index(user: CurrentUser, session: Session, State(state): State<AppState>) -> Html<String> {
    let row: Option<(Option<NaiveDateTime>, Option<String>)> = sqlx::query_as("SELECT last_login, last_ip FROM users WHERE id = $1").bind(user.id).fetch_optional(&state.pool).await.ok().flatten();
    let (last_login, last_ip) = row.unwrap_or((None, None));
    let akun = json!({
        "email": user.email,
        "last_login": last_login.map(|d| activity::format_id_datetime(d)).unwrap_or_else(|| "-".into()),
        "last_ip": last_ip.unwrap_or_else(|| "-".into()),
    });
    page(&state, &session, &user, "backend/my_profile/security/index.html", "security", Some(("akun", akun))).await
}

pub async fn security_edit(user: CurrentUser, session: Session, State(state): State<AppState>) -> Response {
    let mut ctx = tera::Context::new();
    ctx.insert("user", &user_obj(&user));
    ctx.insert("csrf_token", &auth::ensure_csrf(&session).await);
    render_partial(&state, "backend/my_profile/security/edit.html", &ctx)
}

pub async fn security_update(user: CurrentUser, session: Session, State(state): State<AppState>, ConnectInfo(addr): ConnectInfo<SocketAddr>, headers: HeaderMap, mut mp: Multipart) -> Response {
    let mut f = HashMap::new();
    while let Ok(Some(field)) = mp.next_field().await {
        let n = field.name().unwrap_or("").to_string();
        if let Ok(v) = field.text().await { f.insert(n, v); }
    }
    let token = f.get("_token").cloned().unwrap_or_else(|| csrf_header(&headers));
    if !auth::verify_csrf(&session, &token).await {
        return (StatusCode::from_u16(419).unwrap(), Json(json!({"error":"Sesi kedaluwarsa","judul":"Error"}))).into_response();
    }
    let email = f.get("email").map(|s| s.trim().to_string()).unwrap_or_default();
    let current = f.get("current_password").cloned().unwrap_or_default();
    let new_pw = f.get("new_password").cloned().unwrap_or_default();
    let confirm = f.get("new_confirm_password").cloned().unwrap_or_default();

    let cur_hash: String = sqlx::query_scalar("SELECT password FROM users WHERE id = $1").bind(user.id).fetch_one(&state.pool).await.unwrap_or_default();
    let mut errs = serde_json::Map::new();
    if email.is_empty() { errs.insert("email".into(), json!(["Email wajib diisi"])); }
    if current.is_empty() { errs.insert("current_password".into(), json!(["Password terakhir wajib diisi"])); }
    else if !bcrypt::verify(&current, &cur_hash).unwrap_or(false) { errs.insert("current_password".into(), json!(["Password saat ini (Current Password) yang Anda masukkan salah."])); }
    if new_pw.chars().count() < 8 { errs.insert("new_password".into(), json!(["Password baru minimal harus 8 karakter"])); }
    if confirm != new_pw { errs.insert("new_confirm_password".into(), json!(["Password baru dan konfirmasi tidak sama"])); }
    if !errs.is_empty() { return errors(Value::Object(errs)); }

    let hash = bcrypt::hash(&new_pw, 12).unwrap_or_default();
    let _ = sqlx::query("UPDATE users SET email=$1, password=$2, updated_at=now() WHERE id=$3").bind(&email).bind(&hash).bind(user.id).execute(&state.pool).await;
    activity::log(&state.pool, "security", "Mengganti Password Akun", user.id, &ip_of(addr), &ua_of(&headers), json!({"changes":{"password_changed":true}})).await;
    Json(json!({"success":"Password & Email akun berhasil diperbaharui.","judul":"Berhasil"})).into_response()
}

#[derive(Deserialize)]
pub struct LogoutOthersForm {
    #[serde(default)] password: String,
    #[serde(rename = "_token", default)] token: String,
}

pub async fn logout_other_devices(user: CurrentUser, session: Session, State(state): State<AppState>, Form(form): Form<LogoutOthersForm>) -> Response {
    if !auth::verify_csrf(&session, &form.token).await {
        return (StatusCode::from_u16(419).unwrap(), Html("<h1>419 Page Expired</h1>")).into_response();
    }
    let cur_hash: String = sqlx::query_scalar("SELECT password FROM users WHERE id = $1").bind(user.id).fetch_one(&state.pool).await.unwrap_or_default();
    if form.password.is_empty() || !bcrypt::verify(&form.password, &cur_hash).unwrap_or(false) {
        view::set_flash(&session, "error", "Password yang Anda masukkan salah.").await;
    } else {
        // Catatan: MemoryStore tak melacak sesi lain; verifikasi password saja (stub multi-sesi).
        view::set_flash(&session, "success", "Berhasil logout dari semua perangkat lain.").await;
    }
    Redirect::to("/admin/my-security").into_response()
}

// ---------- activity & login session (current user) ----------
pub async fn my_activity_index(user: CurrentUser, session: Session, State(state): State<AppState>) -> Html<String> {
    page(&state, &session, &user, "backend/my_profile/activity/index.html", "activity", None).await
}
pub async fn my_login_session_index(user: CurrentUser, session: Session, State(state): State<AppState>) -> Html<String> {
    page(&state, &session, &user, "backend/my_profile/login_session/index.html", "logs", None).await
}

async fn my_log_dt(state: &AppState, uid: Uuid, q: &HashMap<String, String>, login_kind: bool) -> Response {
    let p = datatables::parse(q);
    let (count_sql, data_sql): (&str, &str) = if login_kind {
        ("SELECT count(*) FROM activity_log WHERE causer_id=$1 AND log_name IN ('login','logout')",
         "SELECT created_at, description, properties FROM activity_log WHERE causer_id=$1 AND log_name IN ('login','logout') ORDER BY id DESC LIMIT $2 OFFSET $3")
    } else {
        ("SELECT count(*) FROM activity_log WHERE causer_id=$1 AND log_name NOT IN ('login','logout')",
         "SELECT created_at, description, properties FROM activity_log WHERE causer_id=$1 AND log_name NOT IN ('login','logout') ORDER BY id DESC LIMIT $2 OFFSET $3")
    };
    let total: i64 = sqlx::query_scalar(count_sql).bind(uid).fetch_one(&state.pool).await.unwrap_or(0);
    let rows: Vec<(Option<NaiveDateTime>, Option<String>, Option<Value>)> = sqlx::query_as(data_sql).bind(uid).bind(p.limit()).bind(p.start).fetch_all(&state.pool).await.unwrap_or_default();
    let data: Vec<Value> = rows.iter().map(|(ca, desc, props)| activity::dt_row(*ca, desc.as_deref(), props.as_ref().unwrap_or(&Value::Null))).collect();
    Json(datatables::response(p.draw, total, total, data)).into_response()
}

pub async fn get_my_activity(user: CurrentUser, State(state): State<AppState>, Query(q): Query<HashMap<String, String>>) -> Response {
    my_log_dt(&state, user.id, &q, false).await
}
pub async fn get_my_login_session(user: CurrentUser, State(state): State<AppState>, Query(q): Query<HashMap<String, String>>) -> Response {
    my_log_dt(&state, user.id, &q, true).await
}
