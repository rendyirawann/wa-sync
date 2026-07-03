use std::collections::HashSet;

use axum::{
    extract::{FromRequestParts, Request},
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Redirect, Response},
};
use sqlx::types::Uuid;
use tower_sessions::Session;

use crate::{auth::SESSION_USER_KEY, AppState};

/// User yang sedang login beserta role & permission efektifnya.
#[derive(Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub avatar: Option<String>,
    pub no_wa: Option<String>,
    pub roles: Vec<String>,
    pub permissions: HashSet<String>,
}

impl CurrentUser {
    /// Superadmin melewati semua pengecekan izin (mirip Gate::before di Laravel).
    pub fn is_superadmin(&self) -> bool {
        self.roles.iter().any(|r| r.eq_ignore_ascii_case("superadmin"))
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r.eq_ignore_ascii_case(role))
    }

    /// Apakah user boleh melakukan suatu permission (Superadmin selalu boleh).
    pub fn can(&self, permission: &str) -> bool {
        self.is_superadmin() || self.permissions.contains(permission)
    }

    /// Label role utama untuk ditampilkan di navbar/sidebar.
    pub fn role_label(&self) -> String {
        self.roles.first().cloned().unwrap_or_else(|| "User".into())
    }
}

/// Halaman 403 sederhana (tema WA) untuk middleware gating.
fn forbidden() -> Response {
    let html = r##"<!doctype html><html lang="id"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>403 — Akses ditolak</title>
<style>body{margin:0;font-family:Inter,system-ui,sans-serif;background:#f4f6fa;color:#1a1a27;display:grid;place-items:center;min-height:100vh}
.box{background:#fff;border-radius:16px;box-shadow:0 6px 24px rgba(20,25,50,.08);padding:48px 40px;text-align:center;max-width:440px}
.ic{width:72px;height:72px;border-radius:18px;background:#fceaf0;color:#f1416c;display:grid;place-items:center;margin:0 auto 20px;font-size:34px;font-weight:800}
h1{font-size:22px;margin:0 0 8px}p{color:#4b5260;line-height:1.6;margin:0 0 22px}
a{display:inline-block;background:#15a85b;color:#fff;font-weight:600;padding:11px 20px;border-radius:9px;text-decoration:none}</style>
</head><body><div class="box"><div class="ic">403</div><h1>Akses ditolak</h1>
<p>Akun Anda tidak memiliki izin untuk membuka halaman ini. Hubungi Superadmin bila ini keliru.</p>
<a href="/admin/dashboard">Kembali ke Dashboard</a></div></body></html>"##;
    (StatusCode::FORBIDDEN, Html(html)).into_response()
}

/// Middleware gating: butuh permission `view_resources` (kelola User/Role/Settings).
/// Superadmin bypass otomatis. Role "User" tidak punya izin ini → 403.
pub async fn guard_resources(user: CurrentUser, req: Request, next: Next) -> Response {
    if user.can("view_resources") {
        next.run(req).await
    } else {
        forbidden()
    }
}

/// Muat role + permission efektif (lewat role maupun langsung) dari tabel spatie.
pub async fn load_current_user(state: &AppState, user_id: Uuid) -> Option<CurrentUser> {
    let (name, email, avatar, no_wa): (String, String, Option<String>, Option<String>) =
        sqlx::query_as("SELECT name, email, avatar, no_wa FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&state.pool)
            .await
            .ok()
            .flatten()?;

    let roles: Vec<String> = sqlx::query_scalar(
        "SELECT r.name FROM roles r \
         JOIN model_has_roles mhr ON mhr.role_id = r.id \
         WHERE mhr.model_id = $1",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let permissions: Vec<String> = sqlx::query_scalar(
        "SELECT p.name FROM permissions p \
         JOIN role_has_permissions rhp ON rhp.permission_id = p.id \
         JOIN model_has_roles mhr ON mhr.role_id = rhp.role_id \
         WHERE mhr.model_id = $1 \
         UNION \
         SELECT p.name FROM permissions p \
         JOIN model_has_permissions mhp ON mhp.permission_id = p.id \
         WHERE mhp.model_id = $1",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    Some(CurrentUser {
        id: user_id,
        name,
        email,
        avatar,
        no_wa,
        roles,
        permissions: permissions.into_iter().collect(),
    })
}

/// Extractor: memuat CurrentUser dari session. Redirect ke login bila belum auth.
impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let to_login = || Redirect::to("/admin/login").into_response();

        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| to_login())?;

        let uid: Option<String> = session.get(SESSION_USER_KEY).await.unwrap_or(None);
        let Some(uid) = uid else {
            return Err(to_login());
        };
        let Ok(user_id) = Uuid::parse_str(&uid) else {
            return Err(to_login());
        };

        load_current_user(state, user_id).await.ok_or_else(to_login)
    }
}
