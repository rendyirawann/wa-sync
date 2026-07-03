use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use serde_json::Value;
use sqlx::types::Uuid;
use tower_sessions::Session;

use crate::{auth, settings, view, AppState};

/// (auth_url, token_url, userinfo_url, scope) per provider.
fn endpoints(p: &str) -> Option<(&'static str, &'static str, &'static str, &'static str)> {
    match p {
        "google" => Some((
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
            "https://www.googleapis.com/oauth2/v2/userinfo",
            "openid email profile",
        )),
        "facebook" => Some((
            "https://www.facebook.com/v18.0/dialog/oauth",
            "https://graph.facebook.com/v18.0/oauth/access_token",
            "https://graph.facebook.com/me?fields=id,name,email",
            "email public_profile",
        )),
        "github" => Some((
            "https://github.com/login/oauth/authorize",
            "https://github.com/login/oauth/access_token",
            "https://api.github.com/user",
            "read:user user:email",
        )),
        "linkedin" => Some((
            "https://www.linkedin.com/oauth/v2/authorization",
            "https://www.linkedin.com/oauth/v2/accessToken",
            "https://api.linkedin.com/v2/userinfo",
            "openid email profile",
        )),
        _ => None,
    }
}

/// driver "linkedin-openid" → key "linkedin".
fn norm(p: &str) -> String {
    p.trim_end_matches("-openid").to_string()
}

fn host_of(h: &HeaderMap) -> String {
    h.get("host").and_then(|v| v.to_str().ok()).unwrap_or("127.0.0.1:8090").to_string()
}

/// URL-encode minimal untuk query param.
fn enc(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{:02X}", b),
        })
        .collect()
}

async fn fail(session: &Session, msg: &str) -> Response {
    view::set_flash(session, "error", msg).await;
    Redirect::to("/admin/login").into_response()
}

/// GET /admin/auth/{provider}/redirect
pub async fn redirect(State(state): State<AppState>, session: Session, headers: HeaderMap, Path(provider): Path<String>) -> Response {
    let key = norm(&provider);
    let s = settings::all(&state.pool).await;
    if settings::get(&s, &format!("social_{key}_enabled"), "0") != "1" {
        return fail(&session, "Login via provider ini tidak aktif.").await;
    }
    let Some((auth_url, _, _, scope)) = endpoints(&key) else {
        return fail(&session, "Provider tidak dikenal.").await;
    };
    let client_id = settings::get(&s, &format!("social_{key}_client_id"), "").to_string();
    if client_id.is_empty() {
        return fail(&session, "Konfigurasi social login belum lengkap.").await;
    }
    let state_tok = Uuid::new_v4().to_string();
    let _ = session.insert("oauth_state", &state_tok).await;
    let redirect_uri = format!("http://{}/admin/auth/{}/callback", host_of(&headers), provider);
    let url = format!(
        "{auth_url}?response_type=code&client_id={cid}&redirect_uri={ru}&scope={sc}&state={st}",
        cid = enc(&client_id),
        ru = enc(&redirect_uri),
        sc = enc(scope),
        st = state_tok
    );
    Redirect::to(&url).into_response()
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    #[serde(default)]
    code: String,
    #[serde(default)]
    state: String,
}

/// GET /admin/auth/{provider}/callback
pub async fn callback(State(state): State<AppState>, session: Session, headers: HeaderMap, Path(provider): Path<String>, Query(q): Query<CallbackQuery>) -> Response {
    let key = norm(&provider);
    let saved: Option<String> = session.get("oauth_state").await.ok().flatten();
    if q.code.is_empty() || saved.as_deref() != Some(q.state.as_str()) {
        return fail(&session, "Sesi OAuth tidak valid.").await;
    }
    let _ = session.remove::<String>("oauth_state").await;

    let s = settings::all(&state.pool).await;
    let Some((_, token_url, userinfo_url, _)) = endpoints(&key) else {
        return fail(&session, "Provider tidak dikenal.").await;
    };
    let client_id = settings::get(&s, &format!("social_{key}_client_id"), "").to_string();
    let client_secret = settings::get(&s, &format!("social_{key}_client_secret"), "").to_string();
    let redirect_uri = format!("http://{}/admin/auth/{}/callback", host_of(&headers), provider);

    let client = reqwest::Client::new();
    let token_resp = client
        .post(token_url)
        .header("Accept", "application/json")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", q.code.as_str()),
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send()
        .await;
    let Ok(resp) = token_resp else { return fail(&session, "Gagal menukar token.").await; };
    let Ok(tok): Result<Value, _> = resp.json().await else { return fail(&session, "Respon token tidak valid.").await; };
    let Some(access) = tok.get("access_token").and_then(Value::as_str) else {
        return fail(&session, "Token akses tidak ditemukan.").await;
    };

    let info_resp = client
        .get(userinfo_url)
        .bearer_auth(access)
        .header("User-Agent", "wa-sync")
        .header("Accept", "application/json")
        .send()
        .await;
    let Ok(iresp) = info_resp else { return fail(&session, "Gagal ambil data profil.").await; };
    let Ok(info): Result<Value, _> = iresp.json().await else { return fail(&session, "Profil tidak valid.").await; };

    let social_id = info
        .get("id")
        .map(|v| v.as_str().map(String::from).unwrap_or_else(|| v.to_string()))
        .or_else(|| info.get("sub").and_then(Value::as_str).map(String::from))
        .unwrap_or_default();
    let email = info.get("email").and_then(Value::as_str).unwrap_or("").to_lowercase();
    let name = info
        .get("name")
        .and_then(Value::as_str)
        .or_else(|| info.get("login").and_then(Value::as_str))
        .unwrap_or("User")
        .to_string();
    if email.is_empty() && social_id.is_empty() {
        return fail(&session, "Email/ID tidak tersedia dari provider.").await;
    }

    let existing: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email = $1 OR (social_id = $2 AND social_type = $3) LIMIT 1")
        .bind(&email)
        .bind(&social_id)
        .bind(&key)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten();

    let user_id = if let Some(uid) = existing {
        let _ = sqlx::query("UPDATE users SET social_id = $1, social_type = $2, last_login = now() WHERE id = $3")
            .bind(&social_id).bind(&key).bind(uid).execute(&state.pool).await;
        uid
    } else {
        let uid = Uuid::now_v7();
        let username = format!("{}-{}", key, &uid.to_string()[..8]);
        let randpw = bcrypt::hash(Uuid::new_v4().to_string(), 12).unwrap_or_default();
        let em = if email.is_empty() { format!("{social_id}@{key}.social") } else { email.clone() };
        let res = sqlx::query("INSERT INTO users (id, name, username, email, password, social_id, social_type, is_active, last_login, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,true,now(),now(),now())")
            .bind(uid).bind(&name).bind(&username).bind(&em).bind(&randpw).bind(&social_id).bind(&key)
            .execute(&state.pool).await;
        if res.is_err() {
            return fail(&session, "Gagal membuat akun.").await;
        }
        // Assign role 'admin' bila ada (hindari Superadmin otomatis).
        if let Some(rid) = sqlx::query_scalar::<_, i64>("SELECT id FROM roles WHERE name = 'admin' LIMIT 1").fetch_optional(&state.pool).await.ok().flatten() {
            let _ = sqlx::query("INSERT INTO model_has_roles (role_id, model_type, model_id) VALUES ($1, 'App\\Models\\User', $2) ON CONFLICT DO NOTHING")
                .bind(rid).bind(uid).execute(&state.pool).await;
        }
        uid
    };
    let _ = session.insert(auth::SESSION_USER_KEY, user_id.to_string()).await;
    Redirect::to("/admin/dashboard").into_response()
}
