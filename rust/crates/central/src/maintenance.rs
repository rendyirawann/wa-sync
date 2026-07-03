use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Html, IntoResponse, Response},
};
use sqlx::types::Uuid;
use tower_sessions::Session;

use crate::{auth::SESSION_USER_KEY, rbac, settings, AppState};

/// Middleware maintenance: bila `maintenance_mode=1`, hanya Superadmin yang boleh lewat;
/// lainnya dapat halaman 503. (Login/logout di luar router ini, jadi tetap bisa diakses.)
pub async fn guard(State(state): State<AppState>, session: Session, req: Request, next: Next) -> Response {
    let s = settings::all(&state.pool).await;
    if settings::get(&s, "maintenance_mode", "0") != "1" {
        return next.run(req).await;
    }
    // Maintenance ON → Superadmin bypass.
    if let Ok(Some(uid)) = session.get::<String>(SESSION_USER_KEY).await {
        if let Ok(user_id) = Uuid::parse_str(&uid) {
            if let Some(u) = rbac::load_current_user(&state, user_id).await {
                if u.is_superadmin() {
                    return next.run(req).await;
                }
            }
        }
    }
    let mut ctx = tera::Context::new();
    ctx.insert("site_name", settings::get(&s, "site_name", "StarterTemp"));
    ctx.insert("site_logo", settings::get(&s, "site_logo", "base-logo.png"));
    match state.tera.render("errors/503.html", &ctx) {
        Ok(html) => (StatusCode::SERVICE_UNAVAILABLE, Html(html)).into_response(),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, Html("<h1>503 Service Unavailable</h1>".to_string())).into_response(),
    }
}
