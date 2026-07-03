use tera::Context;
use tower_sessions::Session;

use crate::{auth, rbac::CurrentUser, settings, AppState};

/// Context dasar untuk semua halaman admin (shell Metronic):
/// app settings, info user, flag Superadmin (untuk menu), token CSRF.
/// `active` = penanda menu aktif (mis. "dashboard", "users", "roles", "account", "settings", "log-activity").
pub async fn base_context(
    state: &AppState,
    session: &Session,
    user: &CurrentUser,
    active: &str,
) -> Context {
    let mut ctx = Context::new();
    let app_settings = settings::all(&state.pool).await;

    // Shortcut yang dipakai di <head> & layout (hindari akses map yang bisa undefined di Tera).
    ctx.insert("site_name", settings::get(&app_settings, "site_name", "StarterTemp"));
    ctx.insert("site_logo", settings::get(&app_settings, "site_logo", "base-logo.png"));
    ctx.insert("site_font", settings::get(&app_settings, "site_font", "Plus Jakarta Sans"));
    ctx.insert("app_settings", &app_settings);

    ctx.insert("active", active);
    ctx.insert("current_url", "");

    ctx.insert("user_id", &user.id.to_string());
    ctx.insert("user_name", &user.name);
    ctx.insert("user_email", &user.email);
    ctx.insert("user_avatar", user.avatar.as_deref().unwrap_or("default.png"));
    ctx.insert("user_no_wa", user.no_wa.as_deref().unwrap_or(""));
    ctx.insert("user_role", &user.role_label());
    ctx.insert("is_superadmin", &user.is_superadmin());
    // Flag izin untuk pembedaan menu/dashboard per role (Superadmin bypass otomatis di can()).
    ctx.insert("can_view_resources", &user.can("view_resources"));

    ctx.insert("csrf_token", &auth::ensure_csrf(session).await);

    // Menu sidebar dinamis dari tabel `menus` (sudah difilter per-izin user).
    ctx.insert("menu_groups", &crate::menu::load(&state.pool, user).await);

    // Flash messages (sekali tampil): baca dari session lalu hapus.
    for kind in ["success", "error", "warning", "info"] {
        let key = format!("flash_{kind}");
        let msg: String = session.get::<String>(&key).await.ok().flatten().unwrap_or_default();
        if !msg.is_empty() {
            let _ = session.remove::<String>(&key).await;
        }
        ctx.insert(format!("flash_{kind}"), &msg);
    }
    ctx
}

/// Set flash message (ditampilkan toastr di render berikutnya).
pub async fn set_flash(session: &Session, kind: &str, msg: &str) {
    let _ = session.insert(&format!("flash_{kind}"), msg).await;
}
