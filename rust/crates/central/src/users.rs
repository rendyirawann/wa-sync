use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{
    extract::{ConnectInfo, Multipart, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::types::Uuid;

use crate::{activity, auth, ban, datatables, rbac::CurrentUser, view, AppState};

// ===================== util =====================

fn ip_of(addr: SocketAddr) -> String {
    addr.ip().to_string()
}
fn ua_of(headers: &HeaderMap) -> String {
    headers.get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or("").to_string()
}
fn csrf_header(headers: &HeaderMap) -> String {
    headers.get("x-csrf-token").and_then(|v| v.to_str().ok()).unwrap_or("").to_string()
}
fn errors_json(map: Value) -> Response {
    Json(json!({ "errors": map })).into_response()
}
fn now_ts() -> i64 {
    chrono::Utc::now().timestamp()
}
fn avatar_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../storage/app/public/user/avatar")
}

/// Resolusi nilai role (id numerik atau nama) → role_id.
async fn resolve_role_id(state: &AppState, val: &str) -> Option<i64> {
    if let Ok(id) = val.parse::<i64>() {
        let found: Option<i64> = sqlx::query_scalar("SELECT id FROM roles WHERE id = $1")
            .bind(id).fetch_optional(&state.pool).await.ok().flatten();
        if found.is_some() { return found; }
    }
    sqlx::query_scalar("SELECT id FROM roles WHERE name = $1")
        .bind(val).fetch_optional(&state.pool).await.ok().flatten()
}

async fn assign_roles(state: &AppState, user_id: Uuid, roles: &[String]) {
    for r in roles {
        if let Some(rid) = resolve_role_id(state, r).await {
            let _ = sqlx::query(
                "INSERT INTO model_has_roles (role_id, model_type, model_id) \
                 VALUES ($1, 'App\\Models\\User', $2) ON CONFLICT DO NOTHING",
            )
            .bind(rid).bind(user_id).execute(&state.pool).await;
        }
    }
}

struct MultipartForm {
    fields: HashMap<String, String>,
    roles: Vec<String>,
    avatar: Option<(String, Vec<u8>)>, // (filename, bytes)
}

async fn read_multipart(mut mp: Multipart) -> MultipartForm {
    let mut fields = HashMap::new();
    let mut roles = Vec::new();
    let mut avatar = None;
    while let Ok(Some(field)) = mp.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "avatar" {
            let fname = field.file_name().unwrap_or("").to_string();
            let bytes = field.bytes().await.map(|b| b.to_vec()).unwrap_or_default();
            if !fname.is_empty() && !bytes.is_empty() {
                avatar = Some((fname, bytes));
            }
        } else if name == "roles" || name == "roles[]" {
            if let Ok(v) = field.text().await {
                if !v.is_empty() { roles.push(v); }
            }
        } else if let Ok(v) = field.text().await {
            fields.insert(name, v);
        }
    }
    MultipartForm { fields, roles, avatar }
}

fn ext_ok(fname: &str) -> Option<String> {
    let ext = fname.rsplit('.').next().unwrap_or("").to_lowercase();
    if ["jpg", "jpeg", "png", "svg"].contains(&ext.as_str()) { Some(ext) } else { None }
}

/// Simpan avatar ke storage/app/public/user/avatar; balikan nama file.
fn save_avatar(user_id: Uuid, fname: &str, bytes: &[u8]) -> Option<String> {
    let ext = ext_ok(fname)?;
    let dir = avatar_dir();
    let _ = std::fs::create_dir_all(&dir);
    let filename = format!("avatar-{}-{}.{}", user_id, now_ts(), ext);
    std::fs::write(dir.join(&filename), bytes).ok()?;
    Some(filename)
}

fn delete_avatar(name: &str) {
    if !name.is_empty() {
        let _ = std::fs::remove_file(avatar_dir().join(name));
    }
}

fn render_partial(state: &AppState, tpl: &str, ctx: &tera::Context) -> Response {
    match state.tera.render(tpl, ctx) {
        Ok(html) => Json(json!({ "html": html })).into_response(),
        Err(e) => Json(json!({ "html": format!("<pre>Template error: {e:#}</pre>") })).into_response(),
    }
}

// ===================== index =====================

pub async fn index(user: CurrentUser, session: tower_sessions::Session, State(state): State<AppState>) -> Html<String> {
    let roles: Vec<(i64, String)> = sqlx::query_as("SELECT id, name FROM roles ORDER BY id DESC")
        .fetch_all(&state.pool).await.unwrap_or_default();
    let roles_json: Vec<Value> = roles.iter().map(|(id, name)| json!({ "id": id, "name": name })).collect();

    let mut ctx = view::base_context(&state, &session, &user, "users").await;
    ctx.insert("roles", &roles_json);
    ctx.insert("can_user_create", &user.can("user.create"));
    ctx.insert("can_user_show", &user.can("user.show"));
    ctx.insert("can_user_edit", &user.can("user.edit"));
    ctx.insert("can_user_delete", &user.can("user.delete"));
    ctx.insert("can_user_massdelete", &user.can("user.massdelete"));
    ctx.insert("can_user_ban", &user.can("user.ban"));
    match state.tera.render("backend/user_management/user/index.html", &ctx) {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

// ===================== datatable =====================

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    name: String,
    email: String,
    avatar: Option<String>,
    last_ip: Option<String>,
    last_login: Option<NaiveDateTime>,
    created_at: Option<NaiveDateTime>,
    banned_at: Option<NaiveDateTime>,
    role_names: String,
    expired_at: Option<NaiveDateTime>,
}

const USERS_DT_SQL: &str = "SELECT u.id, u.name, u.email, u.avatar, u.last_ip, u.last_login, u.created_at, u.banned_at, \
    COALESCE(string_agg(DISTINCT r.name, ', '), '') AS role_names, lb.expired_at \
 FROM users u \
 LEFT JOIN model_has_roles mhr ON mhr.model_id = u.id \
 LEFT JOIN roles r ON r.id = mhr.role_id \
 LEFT JOIN LATERAL (SELECT expired_at FROM bans b WHERE b.bannable_id = u.id AND b.deleted_at IS NULL ORDER BY b.id DESC LIMIT 1) lb ON true \
 WHERE ($1 = '' OR u.name ILIKE '%'||$1||'%' OR u.nik ILIKE '%'||$1||'%' OR u.email ILIKE '%'||$1||'%') \
   AND ($2 = '' OR u.id IN (SELECT mr.model_id FROM model_has_roles mr WHERE mr.role_id = NULLIF($2,'')::bigint)) \
 GROUP BY u.id, lb.expired_at \
 ORDER BY u.created_at DESC LIMIT $3 OFFSET $4";

const USERS_DT_COUNT_SQL: &str = "SELECT count(DISTINCT u.id) FROM users u \
 WHERE ($1 = '' OR u.name ILIKE '%'||$1||'%' OR u.nik ILIKE '%'||$1||'%' OR u.email ILIKE '%'||$1||'%') \
   AND ($2 = '' OR u.id IN (SELECT mr.model_id FROM model_has_roles mr WHERE mr.role_id = NULLIF($2,'')::bigint))";

pub async fn datatable(
    user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let p = datatables::parse(&q);
    let filterrole = q.get("filterrole").cloned().unwrap_or_default();
    let is_super = user.is_superadmin();

    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM users").fetch_one(&state.pool).await.unwrap_or(0);
    let filtered: i64 = sqlx::query_scalar(USERS_DT_COUNT_SQL)
        .bind(&p.search).bind(&filterrole)
        .fetch_one(&state.pool).await.unwrap_or(0);

    let rows: Vec<UserRow> = sqlx::query_as(USERS_DT_SQL)
        .bind(&p.search).bind(&filterrole).bind(p.limit()).bind(p.start)
        .fetch_all(&state.pool).await.unwrap_or_default();

    let data: Vec<Value> = rows.iter().map(|r| {
        let avatar_html = if let Some(av) = &r.avatar {
            format!(
                "<div class=\"d-flex align-items-center\"><div class=\"symbol symbol-45px me-5\"><img src=\"/storage/user/avatar/{av}\" alt=\"{name}\" /></div><div class=\"d-flex flex-column\"><a href=\"/admin/users/{id}\" class=\"text-gray-800 text-hover-primary mb-1\">{name}</a><span>{email}</span></div></div>",
                av = av, name = r.name, id = r.id, email = r.email
            )
        } else {
            let initial = r.name.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
            format!(
                "<div class=\"d-flex align-items-center\"><div class=\"symbol symbol-circle symbol-50px overflow-hidden me-3\"><div class=\"symbol-label fs-3 bg-light-primary text-primary\">{initial}</div></div><div class=\"d-flex flex-column\"><a href=\"/admin/users/{id}\" class=\"text-gray-800 text-hover-primary mb-1\">{name}</a><span>{email}</span></div></div>",
                initial = initial, id = r.id, name = r.name, email = r.email
            )
        };

        let roles_html = if r.role_names.is_empty() { "no roles assigned".to_string() } else { r.role_names.clone() };

        let last_login_html = match r.last_login {
            Some(dt) => format!("<div class=\"badge badge-light fw-bold\">{}</div>", activity::human_diff(dt)),
            None => "<div class=\"badge badge-light fw-bold\">Never logged in</div>".to_string(),
        };
        let last_ip_html = format!(
            "<div class=\"badge badge-light fw-bold\">{}</div>",
            r.last_ip.clone().filter(|s| !s.is_empty()).unwrap_or_else(|| "N/A".into())
        );
        let joined_html = match r.created_at {
            Some(dt) => format!("<div class=\"badge badge-light fw-bold\">{}</div>", activity::format_id_datetime(dt)),
            None => "<div class=\"badge badge-light fw-bold\">N/A</div>".to_string(),
        };
        let status_html = ban::status_html(r.banned_at, r.expired_at);

        let mut action = String::from(
            "<div class=\"dropdown text-end\"><button class=\"btn btn-sm btn-secondary\" type=\"button\" data-bs-toggle=\"dropdown\" aria-expanded=\"false\">Actions <i class=\"ki-outline ki-down fs-5 ms-1\"></i></button><ul class=\"dropdown-menu dropdown-menu-dark fs-6\">",
        );
        if is_super {
            action.push_str(&format!("<li><a class=\"dropdown-item btn px-3 btn-detail\" href=\"javascript:void(0)\" data-id=\"{id}\" >Detail</a></li>", id = r.id));
            action.push_str(&format!("<li><a class=\"dropdown-item btn px-3 btn-edit\" href=\"javascript:void(0)\" data-id=\"{id}\" >Edit</a></li>", id = r.id));
            action.push_str(&format!("<li><a class=\"dropdown-item btn px-3\" data-id=\"{id}\" data-bs-toggle=\"modal\" data-bs-target=\"#Modal_Hapus_Data\" id=\"getDeleteId\">Hapus</a></li>", id = r.id));
            if ban::is_banned(r.banned_at) {
                action.push_str(&format!("<li><a class=\"dropdown-item px-3 text-success\" href=\"javascript:void(0)\" onclick=\"unbanUser('{id}')\">Unbanned</a></li>", id = r.id));
            } else {
                action.push_str(&format!("<li><a class=\"dropdown-item px-3 text-danger\" href=\"javascript:void(0)\" onclick=\"openBanModal('{id}')\">Banned</a></li>", id = r.id));
            }
        }
        action.push_str("</ul></div>");

        json!({
            "id": r.id,
            "avatar": avatar_html,
            "roles": roles_html,
            "last_login_at": last_login_html,
            "last_login_ip": last_ip_html,
            "joined_date": joined_html,
            "status": status_html,
            "action": action,
        })
    }).collect();

    Json(datatables::response(p.draw, total, filtered, data)).into_response()
}

// ===================== store =====================

pub async fn store(
    user: CurrentUser,
    session: tower_sessions::Session,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    mp: Multipart,
) -> Response {
    let form = read_multipart(mp).await;
    let token = form.fields.get("_token").cloned().unwrap_or_else(|| csrf_header(&headers));
    if !auth::verify_csrf(&session, &token).await {
        return (StatusCode::from_u16(419).unwrap(), Json(json!({"error":"Sesi kedaluwarsa","judul":"Error"}))).into_response();
    }

    let g = |k: &str| form.fields.get(k).map(String::as_str).unwrap_or("").trim().to_string();
    let name = g("name");
    let username = g("username");
    let no_wa = g("no_wa");
    let email = g("email");
    let password = form.fields.get("password").cloned().unwrap_or_default();
    let password_conf = form.fields.get("password_confirmation").cloned().unwrap_or_default();

    let mut errs = serde_json::Map::new();
    if name.is_empty() { errs.insert("name".into(), json!(["Nama Lengkap wajib diisi"])); }
    else if name.chars().count() > 255 { errs.insert("name".into(), json!(["Nama Lengkap maksimal 255 karakter"])); }
    if username.is_empty() { errs.insert("username".into(), json!(["Username wajib diisi."])); }
    else if !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') { errs.insert("username".into(), json!(["Username hanya boleh huruf, angka, strip (-) dan underscore (_)."])); }
    else if exists(&state, "username", &username, None).await { errs.insert("username".into(), json!(["Username sudah digunakan, pilih username lain."])); }
    if no_wa.is_empty() { errs.insert("no_wa".into(), json!(["Nomor WhatsApp wajib diisi."])); }
    else if no_wa.chars().count() < 10 || no_wa.chars().count() > 20 { errs.insert("no_wa".into(), json!(["Nomor WhatsApp 10-20 karakter."])); }
    else if exists(&state, "no_wa", &no_wa, None).await { errs.insert("no_wa".into(), json!(["Nomor WhatsApp sudah digunakan oleh pengguna lain."])); }
    if email.is_empty() { errs.insert("email".into(), json!(["Email wajib diisi"])); }
    else if !email.contains('@') { errs.insert("email".into(), json!(["Format Email tidak valid"])); }
    else if exists(&state, "email", &email, None).await { errs.insert("email".into(), json!(["Email sudah terdaftar"])); }
    if password.chars().count() < 8 { errs.insert("password".into(), json!(["Kata Sandi minimal 8 karakter"])); }
    else if password != password_conf { errs.insert("password".into(), json!(["Kata Sandi tidak sama"])); }
    if form.roles.is_empty() { errs.insert("roles".into(), json!(["Role wajib diisi"])); }
    if let Some((fname, bytes)) = &form.avatar {
        if ext_ok(fname).is_none() { errs.insert("avatar".into(), json!(["Avatar harus format .jpg .png .svg"])); }
        else if bytes.len() > 2 * 1024 * 1024 { errs.insert("avatar".into(), json!(["Ukuran file Avatar maksimal 2 MB"])); }
    }
    if !errs.is_empty() { return errors_json(Value::Object(errs)); }

    let id = Uuid::now_v7();
    let avatar = form.avatar.as_ref().and_then(|(f, b)| save_avatar(id, f, b));
    let hash = bcrypt::hash(&password, 12).unwrap_or_default();

    let res = sqlx::query(
        "INSERT INTO users (id, name, username, email, no_wa, avatar, password, is_active, created_at, updated_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,true,now(),now())",
    )
    .bind(id).bind(&name).bind(&username).bind(&email).bind(&no_wa).bind(&avatar).bind(&hash)
    .execute(&state.pool).await;

    if let Err(e) = res {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"Terjadi kesalahan di aplikasi, hubungi Developer.","judul":"Aplikasi Error","errorMessage":e.to_string()}))).into_response();
    }
    assign_roles(&state, id, &form.roles).await;

    activity::log(&state.pool, "tambah user", &format!("Membuat akun user {name}"), user.id, &ip_of(addr), &ua_of(&headers),
        json!({ "new": { "id": id, "name": name, "username": username, "email": email, "no_wa": no_wa } })).await;

    (StatusCode::CREATED, Json(json!({"success":"Data berhasil disimpan.","judul":"Berhasil"}))).into_response()
}

async fn exists(state: &AppState, col: &str, val: &str, except: Option<Uuid>) -> bool {
    let sql = match (col, except.is_some()) {
        ("username", false) => "SELECT 1 FROM users WHERE username = $1",
        ("username", true) => "SELECT 1 FROM users WHERE username = $1 AND id <> $2",
        ("no_wa", false) => "SELECT 1 FROM users WHERE no_wa = $1",
        ("no_wa", true) => "SELECT 1 FROM users WHERE no_wa = $1 AND id <> $2",
        ("email", false) => "SELECT 1 FROM users WHERE email = $1",
        _ => "SELECT 1 FROM users WHERE email = $1 AND id <> $2",
    };
    let mut q = sqlx::query_scalar::<_, i32>(sql).bind(val);
    if let Some(id) = except { q = q.bind(id); }
    q.fetch_optional(&state.pool).await.ok().flatten().is_some()
}

// ===================== edit (form) =====================

pub async fn edit(_user: CurrentUser, session: tower_sessions::Session, State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    let u: Option<(String, String, Option<String>, Option<String>)> =
        sqlx::query_as("SELECT name, email, no_wa, avatar FROM users WHERE id = $1")
            .bind(id).fetch_optional(&state.pool).await.ok().flatten();
    let Some((name, email, no_wa, avatar)) = u else {
        return Json(json!({ "html": "<div class='alert alert-danger'>User tidak ditemukan</div>" })).into_response();
    };
    let user_roles: Vec<String> = sqlx::query_scalar(
        "SELECT r.name FROM roles r JOIN model_has_roles mhr ON mhr.role_id = r.id WHERE mhr.model_id = $1",
    ).bind(id).fetch_all(&state.pool).await.unwrap_or_default();
    let roles: Vec<(i64, String)> = sqlx::query_as("SELECT id, name FROM roles WHERE guard_name = 'web' ORDER BY id")
        .fetch_all(&state.pool).await.unwrap_or_default();
    let roles_json: Vec<Value> = roles.iter().map(|(rid, n)| json!({"id": rid, "name": n})).collect();

    let mut ctx = tera::Context::new();
    ctx.insert("edit_user", &json!({"id": id, "name": name, "email": email, "no_wa": no_wa, "avatar": avatar}));
    ctx.insert("roles", &roles_json);
    ctx.insert("user_roles", &user_roles);
    ctx.insert("csrf_token", &auth::ensure_csrf(&session).await);
    render_partial(&state, "backend/user_management/user/edit.html", &ctx)
}

// ===================== update =====================

pub async fn update(
    user: CurrentUser,
    session: tower_sessions::Session,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    mp: Multipart,
) -> Response {
    let form = read_multipart(mp).await;
    let token = form.fields.get("_token").cloned().unwrap_or_else(|| csrf_header(&headers));
    if !auth::verify_csrf(&session, &token).await {
        return (StatusCode::from_u16(419).unwrap(), Json(json!({"error":"Sesi kedaluwarsa","judul":"Error"}))).into_response();
    }
    let g = |k: &str| form.fields.get(k).map(String::as_str).unwrap_or("").trim().to_string();
    let name = g("name");
    let email = g("email");
    let password = form.fields.get("password").cloned().unwrap_or_default();
    let password_conf = form.fields.get("password_confirmation").cloned().unwrap_or_default();

    let old: Option<(String, String, Option<String>)> =
        sqlx::query_as("SELECT name, email, avatar FROM users WHERE id = $1")
            .bind(id).fetch_optional(&state.pool).await.ok().flatten();
    let Some((old_name, old_email, old_avatar)) = old else {
        return (StatusCode::NOT_FOUND, Json(json!({"error":"User tidak ditemukan","judul":"Gagal"}))).into_response();
    };

    let mut errs = serde_json::Map::new();
    if name.is_empty() { errs.insert("name".into(), json!(["Nama Lengkap wajib diisi"])); }
    if email.is_empty() { errs.insert("email".into(), json!(["Email wajib diisi"])); }
    else if !email.contains('@') { errs.insert("email".into(), json!(["Format Email tidak valid"])); }
    else if exists(&state, "email", &email, Some(id)).await { errs.insert("email".into(), json!(["Email sudah terdaftar"])); }
    if !password.is_empty() && password != password_conf { errs.insert("password".into(), json!(["Kata Sandi tidak sama"])); }
    if form.roles.is_empty() { errs.insert("roles".into(), json!(["Role wajib diisi"])); }
    if let Some((fname, bytes)) = &form.avatar {
        if ext_ok(fname).is_none() { errs.insert("avatar".into(), json!(["Avatar harus format .jpg .png .svg"])); }
        else if bytes.len() > 2 * 1024 * 1024 { errs.insert("avatar".into(), json!(["Ukuran file Avatar maksimal 2 MB"])); }
    }
    if !errs.is_empty() { return errors_json(Value::Object(errs)); }

    let mut new_avatar = old_avatar.clone();
    if let Some((fname, bytes)) = &form.avatar {
        if let Some(saved) = save_avatar(id, fname, bytes) {
            if let Some(o) = &old_avatar { delete_avatar(o); }
            new_avatar = Some(saved);
        }
    }

    let _ = sqlx::query("UPDATE users SET name=$1, email=$2, avatar=$3, updated_at=now() WHERE id=$4")
        .bind(&name).bind(&email).bind(&new_avatar).bind(id).execute(&state.pool).await;
    if !password.is_empty() {
        let hash = bcrypt::hash(&password, 12).unwrap_or_default();
        let _ = sqlx::query("UPDATE users SET password=$1 WHERE id=$2").bind(&hash).bind(id).execute(&state.pool).await;
    }
    // sync roles
    let _ = sqlx::query("DELETE FROM model_has_roles WHERE model_id = $1").bind(id).execute(&state.pool).await;
    assign_roles(&state, id, &form.roles).await;

    activity::log(&state.pool, "edit user", &format!("Mengubah akun user {name}"), user.id, &ip_of(addr), &ua_of(&headers),
        json!({ "old": {"name": old_name, "email": old_email}, "new": {"name": name, "email": email} })).await;

    Json(json!({"success":"Data berhasil diperbaharui.","judul":"Berhasil"})).into_response()
}

// ===================== destroy =====================

pub async fn destroy(
    user: CurrentUser,
    session: tower_sessions::Session,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Response {
    if !auth::verify_csrf(&session, &csrf_header(&headers)).await {
        return (StatusCode::from_u16(419).unwrap(), Json(json!({"error":"Sesi kedaluwarsa","judul":"Error"}))).into_response();
    }
    let row: Option<(String, Option<String>)> = sqlx::query_as("SELECT name, avatar FROM users WHERE id = $1")
        .bind(id).fetch_optional(&state.pool).await.ok().flatten();
    let Some((name, avatar)) = row else {
        return Json(json!({"error":"Data tidak ditemukan","judul":"Gagal"})).into_response();
    };
    if let Some(av) = &avatar { delete_avatar(av); }
    let _ = sqlx::query("DELETE FROM model_has_roles WHERE model_id = $1").bind(id).execute(&state.pool).await;
    let _ = sqlx::query("DELETE FROM model_has_permissions WHERE model_id = $1").bind(id).execute(&state.pool).await;
    let _ = sqlx::query("DELETE FROM bans WHERE bannable_id = $1").bind(id).execute(&state.pool).await;
    let _ = sqlx::query("DELETE FROM users WHERE id = $1").bind(id).execute(&state.pool).await;

    activity::log(&state.pool, "hapus user", &format!("Menghapus akun user {name}"), user.id, &ip_of(addr), &ua_of(&headers),
        json!({ "get": {"name": name} })).await;

    Json(json!({"success":"Data berhasil dihapus","judul":"Berhasil"})).into_response()
}

// ===================== mass delete =====================

#[derive(Deserialize)]
pub struct MassDeletePayload {
    #[serde(default)]
    ids: Vec<String>,
    #[serde(rename = "_token", default)]
    token: String,
}

pub async fn mass_delete(
    user: CurrentUser,
    session: tower_sessions::Session,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<MassDeletePayload>,
) -> Response {
    if !auth::verify_csrf(&session, &payload.token).await {
        return (StatusCode::from_u16(419).unwrap(), Json(json!({"status":"error","message":"Sesi kedaluwarsa"}))).into_response();
    }
    if payload.ids.is_empty() {
        return Json(json!({"status":"error","message":"No users selected for deletion."})).into_response();
    }
    let mut count = 0;
    for raw in &payload.ids {
        let Ok(id) = Uuid::parse_str(raw) else { continue };
        let row: Option<(String, Option<String>)> = sqlx::query_as("SELECT name, avatar FROM users WHERE id = $1")
            .bind(id).fetch_optional(&state.pool).await.ok().flatten();
        let Some((name, avatar)) = row else { continue };
        if let Some(av) = &avatar { delete_avatar(av); }
        let _ = sqlx::query("DELETE FROM model_has_roles WHERE model_id = $1").bind(id).execute(&state.pool).await;
        let _ = sqlx::query("DELETE FROM model_has_permissions WHERE model_id = $1").bind(id).execute(&state.pool).await;
        let _ = sqlx::query("DELETE FROM bans WHERE bannable_id = $1").bind(id).execute(&state.pool).await;
        let _ = sqlx::query("DELETE FROM users WHERE id = $1").bind(id).execute(&state.pool).await;
        activity::log(&state.pool, "massdelete user", &format!("Menghapus akun user {name}"), user.id, &ip_of(addr), &ua_of(&headers), json!({"get":{"name":name}})).await;
        count += 1;
    }
    Json(json!({"status":"success","message": format!("{count} users deleted successfully!")})).into_response()
}

// ===================== ban / unban =====================

#[derive(Deserialize)]
pub struct BanPayload {
    #[serde(default)]
    reason: String,
    #[serde(default)]
    duration: String,
    #[serde(rename = "_token", default)]
    token: String,
}

pub async fn ban_user(
    user: CurrentUser,
    session: tower_sessions::Session,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<BanPayload>,
) -> Response {
    if !auth::verify_csrf(&session, &payload.token).await {
        return (StatusCode::from_u16(419).unwrap(), Json(json!({"error":"Sesi kedaluwarsa"}))).into_response();
    }
    if payload.reason.trim().is_empty() || payload.reason.chars().count() > 500 {
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"error":"Alasan wajib diisi (maks 500 karakter)."}))).into_response();
    }
    if !["permanent", "1h", "24h", "1w"].contains(&payload.duration.as_str()) {
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"error":"Durasi tidak valid."}))).into_response();
    }
    if id == user.id {
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"error":"Anda tidak dapat memban akun Anda sendiri."}))).into_response();
    }
    let row: Option<(Option<NaiveDateTime>,)> = sqlx::query_as("SELECT banned_at FROM users WHERE id = $1")
        .bind(id).fetch_optional(&state.pool).await.ok().flatten();
    match row {
        None => return (StatusCode::NOT_FOUND, Json(json!({"error":"User tidak ditemukan."}))).into_response(),
        Some((Some(_),)) => return (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"error":"User sudah diban sebelumnya."}))).into_response(),
        _ => {}
    }
    if ban::ban(&state.pool, id, user.id, &payload.reason, &payload.duration).await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"Gagal memban user."}))).into_response();
    }
    let uname: String = sqlx::query_scalar("SELECT name FROM users WHERE id=$1").bind(id).fetch_one(&state.pool).await.unwrap_or_default();
    activity::log(&state.pool, "ban user", &format!("Membanned user: {uname}"), user.id, &ip_of(addr), &ua_of(&headers),
        json!({"reason": payload.reason, "duration": payload.duration})).await;
    Json(json!({"success":"User berhasil diban"})).into_response()
}

#[derive(Deserialize)]
pub struct UnbanPayload {
    #[serde(rename = "_token", default)]
    token: String,
}

pub async fn unban_user(
    user: CurrentUser,
    session: tower_sessions::Session,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(payload): Json<UnbanPayload>,
) -> Response {
    if !auth::verify_csrf(&session, &payload.token).await {
        return (StatusCode::from_u16(419).unwrap(), Json(json!({"error":"Sesi kedaluwarsa"}))).into_response();
    }
    let row: Option<(Option<NaiveDateTime>,)> = sqlx::query_as("SELECT banned_at FROM users WHERE id = $1")
        .bind(id).fetch_optional(&state.pool).await.ok().flatten();
    if !matches!(row, Some((Some(_),))) {
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"error":"User tidak dalam status banned."}))).into_response();
    }
    if ban::unban(&state.pool, id).await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"Gagal."}))).into_response();
    }
    let uname: String = sqlx::query_scalar("SELECT name FROM users WHERE id=$1").bind(id).fetch_one(&state.pool).await.unwrap_or_default();
    activity::log(&state.pool, "unban user", &format!("Mengaktifkan kembali user: {uname}"), user.id, &ip_of(addr), &ua_of(&headers), json!({})).await;
    Json(json!({"success":"User berhasil diaktifkan kembali"})).into_response()
}

// ===================== show (detail) =====================

pub async fn show(_user: CurrentUser, session: tower_sessions::Session, State(state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    let u: Option<(String, String, Option<String>, Option<String>, Option<NaiveDateTime>, Option<String>)> =
        sqlx::query_as("SELECT name, email, no_wa, avatar, last_login, last_ip FROM users WHERE id = $1")
            .bind(id).fetch_optional(&state.pool).await.ok().flatten();
    let Some((name, email, no_wa, avatar, last_login, last_ip)) = u else {
        return Json(json!({"html":"<div class='alert alert-danger'>User tidak ditemukan</div>"})).into_response();
    };
    let role_names: Vec<String> = sqlx::query_scalar(
        "SELECT r.name FROM roles r JOIN model_has_roles mhr ON mhr.role_id = r.id WHERE mhr.model_id = $1",
    ).bind(id).fetch_all(&state.pool).await.unwrap_or_default();
    let total_activity: i64 = sqlx::query_scalar("SELECT count(*) FROM activity_log WHERE causer_id = $1").bind(id).fetch_one(&state.pool).await.unwrap_or(0);
    let total_login: i64 = sqlx::query_scalar("SELECT count(*) FROM activity_log WHERE causer_id = $1 AND log_name = 'login'").bind(id).fetch_one(&state.pool).await.unwrap_or(0);

    let roles_count = role_names.len();
    let mut ctx = tera::Context::new();
    ctx.insert("data", &json!({
        "id": id, "name": name, "email": email, "no_wa": no_wa, "avatar": avatar,
        "last_ip": last_ip,
        "last_login": last_login.map(activity::format_id_datetime),
        "roles": role_names,
        "roles_count": roles_count,
    }));
    ctx.insert("total_activity", &total_activity);
    ctx.insert("total_login", &total_login);
    render_partial(&state, "backend/user_management/user/show.html", &ctx)
}

// ===================== per-user log datatables =====================

async fn log_datatable(state: &AppState, id: Uuid, q: &HashMap<String, String>, login_kind: bool) -> Response {
    let p = datatables::parse(q);
    let (count_sql, data_sql): (&str, &str) = if login_kind {
        (
            "SELECT count(*) FROM activity_log WHERE causer_id = $1 AND log_name IN ('login','logout')",
            "SELECT created_at, description, properties FROM activity_log WHERE causer_id = $1 AND log_name IN ('login','logout') ORDER BY id DESC LIMIT $2 OFFSET $3",
        )
    } else {
        (
            "SELECT count(*) FROM activity_log WHERE causer_id = $1 AND log_name NOT IN ('login','logout')",
            "SELECT created_at, description, properties FROM activity_log WHERE causer_id = $1 AND log_name NOT IN ('login','logout') ORDER BY id DESC LIMIT $2 OFFSET $3",
        )
    };
    let total: i64 = sqlx::query_scalar(count_sql).bind(id).fetch_one(&state.pool).await.unwrap_or(0);
    let rows: Vec<(Option<NaiveDateTime>, Option<String>, Option<Value>)> =
        sqlx::query_as(data_sql).bind(id).bind(p.limit()).bind(p.start).fetch_all(&state.pool).await.unwrap_or_default();
    let data: Vec<Value> = rows.iter().map(|(ca, desc, props)| {
        activity::dt_row(*ca, desc.as_deref(), props.as_ref().unwrap_or(&Value::Null))
    }).collect();
    Json(datatables::response(p.draw, total, total, data)).into_response()
}

pub async fn user_login_sessions(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>, Query(q): Query<HashMap<String, String>>) -> Response {
    log_datatable(&state, id, &q, true).await
}
pub async fn user_activity(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>, Query(q): Query<HashMap<String, String>>) -> Response {
    log_datatable(&state, id, &q, false).await
}
