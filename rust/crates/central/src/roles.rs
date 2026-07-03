use std::collections::HashMap;
use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Multipart, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{activity, auth, datatables, rbac::CurrentUser, view, AppState};

// ---------- util ----------
fn ip_of(addr: SocketAddr) -> String { addr.ip().to_string() }
fn ua_of(h: &HeaderMap) -> String { h.get("user-agent").and_then(|v| v.to_str().ok()).unwrap_or("").to_string() }
fn csrf_header(h: &HeaderMap) -> String { h.get("x-csrf-token").and_then(|v| v.to_str().ok()).unwrap_or("").to_string() }

fn title_case(s: &str) -> String {
    s.replace(['-', '_'], " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Bangun permission_groups: list {category, items:[{id,name}]} (urut per category).
/// `only` = batasi ke set id tertentu (utk show).
async fn permission_groups(state: &AppState, only: Option<&[i64]>) -> Vec<Value> {
    let rows: Vec<(i64, String, Option<String>)> =
        sqlx::query_as("SELECT id, name, category FROM permissions ORDER BY COALESCE(category,'') ASC, id ASC")
            .fetch_all(&state.pool).await.unwrap_or_default();
    let mut groups: Vec<(String, Vec<Value>)> = Vec::new();
    for (id, name, cat) in rows {
        if let Some(ids) = only {
            if !ids.contains(&id) { continue; }
        }
        let c = cat.unwrap_or_default();
        if let Some(g) = groups.iter_mut().find(|(gc, _)| *gc == c) {
            g.1.push(json!({ "id": id, "name": name }));
        } else {
            groups.push((c, vec![json!({ "id": id, "name": name })]));
        }
    }
    groups.into_iter().map(|(category, items)| json!({ "category": category, "items": items })).collect()
}

async fn read_role_form(mut mp: Multipart) -> (HashMap<String, String>, Vec<i64>) {
    let mut fields = HashMap::new();
    let mut permission = Vec::new();
    while let Ok(Some(field)) = mp.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "permission" || name == "permission[]" {
            if let Ok(v) = field.text().await {
                if let Ok(id) = v.parse::<i64>() { permission.push(id); }
            }
        } else if let Ok(v) = field.text().await {
            fields.insert(name, v);
        }
    }
    (fields, permission)
}

async fn name_exists(state: &AppState, name: &str, except: Option<i64>) -> bool {
    let found: Option<i64> = match except {
        Some(id) => sqlx::query_scalar("SELECT id FROM roles WHERE name = $1 AND id <> $2").bind(name).bind(id).fetch_optional(&state.pool).await.ok().flatten(),
        None => sqlx::query_scalar("SELECT id FROM roles WHERE name = $1").bind(name).fetch_optional(&state.pool).await.ok().flatten(),
    };
    found.is_some()
}

async fn sync_permissions(state: &AppState, role_id: i64, perms: &[i64]) {
    for pid in perms {
        let _ = sqlx::query("INSERT INTO role_has_permissions (permission_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(pid).bind(role_id).execute(&state.pool).await;
    }
}

// ---------- index ----------
pub async fn index(user: CurrentUser, session: tower_sessions::Session, State(state): State<AppState>) -> Html<String> {
    let groups = permission_groups(&state, None).await;
    let mut ctx = view::base_context(&state, &session, &user, "roles").await;
    ctx.insert("permission_groups", &groups);
    ctx.insert("can_role_create", &user.can("role.create"));
    ctx.insert("can_role_show", &user.can("role.show"));
    ctx.insert("can_role_edit", &user.can("role.edit"));
    ctx.insert("can_role_delete", &user.can("role.delete"));
    ctx.insert("can_role_massdelete", &user.can("role.massdelete"));
    match state.tera.render("backend/user_management/role/index.html", &ctx) {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

// ---------- datatable ----------
#[derive(sqlx::FromRow)]
struct RoleRow {
    id: i64,
    name: String,
    guard_name: String,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
}

const ROLES_DT_SQL: &str = "SELECT id, name, guard_name, created_at, updated_at FROM roles \
 WHERE ($1 = '' OR name ILIKE '%'||$1||'%') ORDER BY id DESC LIMIT $2 OFFSET $3";
const ROLES_DT_COUNT_SQL: &str = "SELECT count(*) FROM roles WHERE ($1 = '' OR name ILIKE '%'||$1||'%')";

pub async fn datatable(user: CurrentUser, State(state): State<AppState>, Query(q): Query<HashMap<String, String>>) -> Response {
    let p = datatables::parse(&q);
    let is_super = user.is_superadmin();
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM roles").fetch_one(&state.pool).await.unwrap_or(0);
    let filtered: i64 = sqlx::query_scalar(ROLES_DT_COUNT_SQL).bind(&p.search).fetch_one(&state.pool).await.unwrap_or(0);
    let rows: Vec<RoleRow> = sqlx::query_as(ROLES_DT_SQL).bind(&p.search).bind(p.limit()).bind(p.start).fetch_all(&state.pool).await.unwrap_or_default();

    let data: Vec<Value> = rows.iter().map(|r| {
        let name_html = format!("<span class=\"badge badge-light-primary fs-7 fw-bold\">{}</span>", r.name);
        let guard_html = format!("<span class=\"badge badge-light-info fs-7 fw-bold\">{}</span>", r.guard_name);
        let updated = r.updated_at.map(activity::format_id_datetime).unwrap_or_else(|| "-".into());
        let created = r.created_at.map(activity::format_id_datetime).unwrap_or_else(|| "-".into());
        let mut action = String::from("<div class=\"dropdown text-end\"><button class=\"btn btn-sm btn-secondary\" type=\"button\" data-bs-toggle=\"dropdown\" aria-expanded=\"false\">Actions <i class=\"ki-outline ki-down fs-5 ms-1\"></i></button><ul class=\"dropdown-menu dropdown-menu-dark fs-6\">");
        if is_super {
            action.push_str(&format!("<li><a class=\"dropdown-item btn px-3 btn-detail\" href=\"javascript:void(0)\" data-id=\"{id}\">Detail</a></li>", id = r.id));
            action.push_str(&format!("<li><a class=\"dropdown-item btn px-3 btn-edit\" href=\"javascript:void(0)\" data-id=\"{id}\">Edit</a></li>", id = r.id));
            action.push_str(&format!("<li><a class=\"dropdown-item btn px-3\" data-id=\"{id}\" data-bs-toggle=\"modal\" data-bs-target=\"#Modal_Hapus_Data\" id=\"getDeleteId\">Hapus</a></li>", id = r.id));
        }
        action.push_str("</ul></div>");
        json!({ "id": r.id, "name": name_html, "guard_name": guard_html, "updated_at": updated, "created_at": created, "action": action })
    }).collect();

    Json(datatables::response(p.draw, total, filtered, data)).into_response()
}

// ---------- store ----------
pub async fn store(user: CurrentUser, session: tower_sessions::Session, State(state): State<AppState>, ConnectInfo(addr): ConnectInfo<SocketAddr>, headers: HeaderMap, mp: Multipart) -> Response {
    let (fields, perms) = read_role_form(mp).await;
    let token = fields.get("_token").cloned().unwrap_or_else(|| csrf_header(&headers));
    if !auth::verify_csrf(&session, &token).await {
        return (StatusCode::from_u16(419).unwrap(), Json(json!({"error":"Sesi kedaluwarsa","judul":"Error"}))).into_response();
    }
    let name = fields.get("name").map(|s| s.trim().to_string()).unwrap_or_default();
    let mut errs = serde_json::Map::new();
    if name.is_empty() { errs.insert("name".into(), json!(["Nama Hak Akses wajib diisi"])); }
    else if name_exists(&state, &name, None).await { errs.insert("name".into(), json!(["Nama Hak Akses sudah terdaftar"])); }
    if perms.is_empty() { errs.insert("permission".into(), json!(["Permission wajib diisi"])); }
    if !errs.is_empty() { return Json(json!({ "errors": errs })).into_response(); }

    let role_id: Option<i64> = sqlx::query_scalar("INSERT INTO roles (name, guard_name, created_at, updated_at) VALUES ($1,'web',now(),now()) RETURNING id")
        .bind(&name).fetch_one(&state.pool).await.ok();
    let Some(role_id) = role_id else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"Terjadi kesalahan di aplikasi, hubungi Developer.","judul":"Aplikasi Error"}))).into_response();
    };
    sync_permissions(&state, role_id, &perms).await;
    activity::log(&state.pool, "tambah role", &format!("Membuat role {name}"), user.id, &ip_of(addr), &ua_of(&headers), json!({"new":{"id":role_id,"name":name}})).await;
    Json(json!({"success": format!("Data {name} berhasil disimpan."), "judul":"Berhasil"})).into_response()
}

// ---------- edit (form) ----------
pub async fn edit(_user: CurrentUser, session: tower_sessions::Session, State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    let role: Option<(i64, String)> = sqlx::query_as("SELECT id, name FROM roles WHERE id = $1").bind(id).fetch_optional(&state.pool).await.ok().flatten();
    let Some((rid, rname)) = role else {
        return Json(json!({"html":"<div class='alert alert-danger'>Role tidak ditemukan</div>"})).into_response();
    };
    let role_perms: Vec<i64> = sqlx::query_scalar("SELECT permission_id FROM role_has_permissions WHERE role_id = $1").bind(id).fetch_all(&state.pool).await.unwrap_or_default();
    let groups = permission_groups(&state, None).await;
    let mut ctx = tera::Context::new();
    ctx.insert("role", &json!({"id": rid, "name": rname}));
    ctx.insert("permission_groups", &groups);
    ctx.insert("role_permissions", &role_perms);
    ctx.insert("csrf_token", &auth::ensure_csrf(&session).await);
    match state.tera.render("backend/user_management/role/edit.html", &ctx) {
        Ok(html) => Json(json!({ "html": html })).into_response(),
        Err(e) => Json(json!({ "html": format!("<pre>Template error: {e:#}</pre>") })).into_response(),
    }
}

// ---------- update ----------
pub async fn update(user: CurrentUser, session: tower_sessions::Session, State(state): State<AppState>, ConnectInfo(addr): ConnectInfo<SocketAddr>, headers: HeaderMap, Path(id): Path<i64>, mp: Multipart) -> Response {
    let (fields, perms) = read_role_form(mp).await;
    let token = fields.get("_token").cloned().unwrap_or_else(|| csrf_header(&headers));
    if !auth::verify_csrf(&session, &token).await {
        return (StatusCode::from_u16(419).unwrap(), Json(json!({"error":"Sesi kedaluwarsa","judul":"Error"}))).into_response();
    }
    let name = fields.get("name").map(|s| s.trim().to_string()).unwrap_or_default();
    let old: Option<String> = sqlx::query_scalar("SELECT name FROM roles WHERE id = $1").bind(id).fetch_optional(&state.pool).await.ok().flatten();
    let Some(old_name) = old else {
        return (StatusCode::NOT_FOUND, Json(json!({"error":"Role tidak ditemukan","judul":"Gagal"}))).into_response();
    };
    let mut errs = serde_json::Map::new();
    if name.is_empty() { errs.insert("name".into(), json!(["Nama Hak Akses wajib diisi"])); }
    else if name_exists(&state, &name, Some(id)).await { errs.insert("name".into(), json!(["Nama Hak Akses sudah terdaftar"])); }
    if perms.is_empty() { errs.insert("permission".into(), json!(["Permission wajib diisi"])); }
    if !errs.is_empty() { return Json(json!({ "errors": errs })).into_response(); }

    let _ = sqlx::query("UPDATE roles SET name = $1, updated_at = now() WHERE id = $2").bind(&name).bind(id).execute(&state.pool).await;
    let _ = sqlx::query("DELETE FROM role_has_permissions WHERE role_id = $1").bind(id).execute(&state.pool).await;
    sync_permissions(&state, id, &perms).await;
    activity::log(&state.pool, "edit role", &format!("Mengubah role {name}"), user.id, &ip_of(addr), &ua_of(&headers), json!({"old":{"name":old_name},"new":{"name":name}})).await;
    Json(json!({"success": format!("Data {name} berhasil diperbaharui."), "judul":"Berhasil"})).into_response()
}

// ---------- destroy ----------
pub async fn destroy(user: CurrentUser, session: tower_sessions::Session, State(state): State<AppState>, ConnectInfo(addr): ConnectInfo<SocketAddr>, headers: HeaderMap, Path(id): Path<i64>) -> Response {
    if !auth::verify_csrf(&session, &csrf_header(&headers)).await {
        return (StatusCode::from_u16(419).unwrap(), Json(json!({"error":"Sesi kedaluwarsa","judul":"Error"}))).into_response();
    }
    let name: Option<String> = sqlx::query_scalar("SELECT name FROM roles WHERE id = $1").bind(id).fetch_optional(&state.pool).await.ok().flatten();
    let Some(name) = name else {
        return Json(json!({"error":"Data Gagal dihapus","judul":"Gagal","errorMessage":"Role tidak ditemukan"})).into_response();
    };
    if name.eq_ignore_ascii_case("superadmin") {
        return Json(json!({"error":"Role Superadmin tidak dapat dihapus.","judul":"Gagal"})).into_response();
    }
    let _ = sqlx::query("DELETE FROM role_has_permissions WHERE role_id = $1").bind(id).execute(&state.pool).await;
    let _ = sqlx::query("DELETE FROM model_has_roles WHERE role_id = $1").bind(id).execute(&state.pool).await;
    let _ = sqlx::query("DELETE FROM roles WHERE id = $1").bind(id).execute(&state.pool).await;
    activity::log(&state.pool, "hapus role", &format!("Menghapus role {name}"), user.id, &ip_of(addr), &ua_of(&headers), json!({"get":{"name":name}})).await;
    Json(json!({"success": format!("Data {name} berhasil dihapus"), "judul":"Berhasil"})).into_response()
}

// ---------- mass delete ----------
#[derive(Deserialize)]
pub struct MassDeletePayload {
    #[serde(default)] ids: Vec<String>,
    #[serde(rename = "_token", default)] token: String,
}

pub async fn mass_delete(user: CurrentUser, session: tower_sessions::Session, State(state): State<AppState>, ConnectInfo(addr): ConnectInfo<SocketAddr>, headers: HeaderMap, Json(payload): Json<MassDeletePayload>) -> Response {
    if !auth::verify_csrf(&session, &payload.token).await {
        return (StatusCode::from_u16(419).unwrap(), Json(json!({"status":"error","message":"Sesi kedaluwarsa"}))).into_response();
    }
    if payload.ids.is_empty() {
        return Json(json!({"status":"error","message":"No data selected for deletion."})).into_response();
    }
    let mut count = 0;
    for raw in &payload.ids {
        let Ok(id) = raw.parse::<i64>() else { continue };
        let name: Option<String> = sqlx::query_scalar("SELECT name FROM roles WHERE id = $1").bind(id).fetch_optional(&state.pool).await.ok().flatten();
        let Some(name) = name else { continue };
        if name.eq_ignore_ascii_case("superadmin") { continue; }
        let _ = sqlx::query("DELETE FROM role_has_permissions WHERE role_id = $1").bind(id).execute(&state.pool).await;
        let _ = sqlx::query("DELETE FROM model_has_roles WHERE role_id = $1").bind(id).execute(&state.pool).await;
        let _ = sqlx::query("DELETE FROM roles WHERE id = $1").bind(id).execute(&state.pool).await;
        activity::log(&state.pool, "massdelete role", &format!("Menghapus role {name}"), user.id, &ip_of(addr), &ua_of(&headers), json!({"get":{"name":name}})).await;
        count += 1;
    }
    Json(json!({"status":"success","message": format!("{count} data deleted successfully!")})).into_response()
}

// ---------- generate permissions ----------
pub async fn generate_permissions(user: CurrentUser, session: tower_sessions::Session, State(state): State<AppState>, headers: HeaderMap) -> Response {
    let _ = &user;
    if !auth::verify_csrf(&session, &csrf_header(&headers)).await {
        return (StatusCode::from_u16(419).unwrap(), Json(json!({"status":"error","message":"Sesi kedaluwarsa"}))).into_response();
    }
    // Set permission kanonik aplikasi + kategori dari prefix (Rust tak punya route Laravel utk discan).
    const CANONICAL: [&str; 15] = [
        "view_dashboard", "view_data_master", "view_resources", "view_help",
        "user.show", "user.create", "user.edit", "user.delete", "user.massdelete", "user.ban",
        "role.show", "role.create", "role.edit", "role.delete", "role.massdelete",
    ];
    for name in CANONICAL {
        if let Some((prefix, _)) = name.split_once('.') {
            let category = title_case(prefix);
            let _ = sqlx::query(
                "INSERT INTO permissions (name, guard_name, category, created_at, updated_at) VALUES ($1,'web',$2,now(),now()) \
                 ON CONFLICT (name, guard_name) DO UPDATE SET category = EXCLUDED.category, updated_at = now()",
            ).bind(name).bind(&category).execute(&state.pool).await;
        } else {
            let _ = sqlx::query(
                "INSERT INTO permissions (name, guard_name, created_at, updated_at) VALUES ($1,'web',now(),now()) ON CONFLICT (name, guard_name) DO NOTHING",
            ).bind(name).execute(&state.pool).await;
        }
    }
    Json(json!({"status":"success","message":"Permissions berhasil digenerate ulang."})).into_response()
}

// ---------- show (detail) ----------
pub async fn show(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    let role: Option<(String, String)> = sqlx::query_as("SELECT name, guard_name FROM roles WHERE id = $1").bind(id).fetch_optional(&state.pool).await.ok().flatten();
    let Some((rname, guard)) = role else {
        return Json(json!({"html":"<div class='alert alert-danger'>Role tidak ditemukan</div>"})).into_response();
    };
    let role_perms: Vec<i64> = sqlx::query_scalar("SELECT permission_id FROM role_has_permissions WHERE role_id = $1").bind(id).fetch_all(&state.pool).await.unwrap_or_default();
    let groups = permission_groups(&state, Some(&role_perms)).await;
    let mut ctx = tera::Context::new();
    ctx.insert("role", &json!({"name": rname, "guard_name": guard}));
    ctx.insert("permission_groups", &groups);
    match state.tera.render("backend/user_management/role/show.html", &ctx) {
        Ok(html) => Json(json!({ "html": html })).into_response(),
        Err(e) => Json(json!({ "html": format!("<pre>Template error: {e:#}</pre>") })).into_response(),
    }
}

// ---------- select ----------
pub async fn select(_user: CurrentUser, State(state): State<AppState>, Query(q): Query<HashMap<String, String>>) -> Response {
    let rows: Vec<(i64, String)> = match q.get("q").filter(|s| !s.is_empty()) {
        Some(term) => sqlx::query_as("SELECT id, name FROM roles WHERE name ILIKE '%'||$1||'%' ORDER BY id").bind(term).fetch_all(&state.pool).await.unwrap_or_default(),
        None => sqlx::query_as("SELECT id, name FROM roles ORDER BY id LIMIT 30").fetch_all(&state.pool).await.unwrap_or_default(),
    };
    let data: Vec<Value> = rows.iter().map(|(id, name)| json!({"id": id, "name": name})).collect();
    Json(data).into_response()
}
