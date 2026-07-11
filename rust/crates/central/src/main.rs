mod activity;
mod ai;
mod ai_pipeline;
mod auth;
mod authextra;
mod autoreply;
mod ban;
mod billing;
mod broadcast;
mod broadcast_engine;
mod contacts;
mod coupons;
mod dash;
mod datatables;
mod inbox;
mod knowledge;
mod logactivity;
mod mailer;
mod maintenance;
mod menu;
mod midtrans;
mod notify;
mod plans;
mod profile;
mod publicapi;
mod quota;
mod rbac;
mod ratelimit;
mod roles;
mod seed;
mod sessionstore;
mod settings;
mod social;
mod subscribers;
mod users;
mod view;
mod wa;
mod wagw;
mod wainternal;
mod wamigrate;
mod wasessions;
mod webhook;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    middleware::{from_fn, from_fn_with_state},
    response::{Html, IntoResponse},
    routing::{get, post, put},
    Router,
};
use rbac::CurrentUser;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tera::Tera;
use tower_http::services::ServeDir;
use tower_sessions::{Session, SessionManagerLayer};

/// State aplikasi yang dibagi ke semua handler (di-clone murah; pool/tera/limiter pakai Arc internal).
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub tera: Arc<Tera>,
    pub limiter: Arc<ratelimit::RateLimiter>,
    pub wa: Arc<wagw::Gateway>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    // Muat .env dari folder crate dulu (robust terhadap CWD — mis. saat dijalankan
    // dari root project via dev-all.ps1), lalu fallback ke .env di CWD.
    let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    dotenvy::from_path(&env_path).ok();
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL belum di-set (lihat rust/crates/central/.env)");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET TIME ZONE 'Asia/Jakarta'")
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect_lazy(&db_url)?;
    tracing::info!("Pool PostgreSQL siap (lazy, TZ=Asia/Jakarta)");

    notify::init();

    let manifest_dir = env!("CARGO_MANIFEST_DIR").replace('\\', "/");
    let tera = Tera::new(&format!("{manifest_dir}/templates/**/*.html"))?;
    let assets_dir = format!("{manifest_dir}/../../../public/assets");
    let storage_dir = format!("{manifest_dir}/../../../storage/app/public");

    let state = AppState {
        pool,
        tera: Arc::new(tera),
        limiter: Arc::new(ratelimit::RateLimiter::default()),
        wa: Arc::new(wagw::Gateway::from_env()),
    };

    // Session persisten di Postgres (tabel `sessions`) — sesi login bertahan saat restart.
    let store = sessionstore::PgStore::new(state.pool.clone());
    store.migrate().await?;

    // Seeder idempotent: permission + role Superadmin/User + akun contoh.
    seed::run(&state.pool).await;

    // Migrasi tabel WhatsApp (Fase B): wa_sessions, wa_messages.
    wamigrate::run(&state.pool).await?;

    // Menu sidebar dinamis: tabel `menus` + seed default (sekali).
    menu::migrate(&state.pool).await?;
    menu::seed(&state.pool).await;
    menu::ensure(&state.pool).await;

    // Akun subscriber Enterprise (role User, 1 tahun) + data contoh — butuh plans/wa_* dari wamigrate.
    seed::enterprise_subscriber(&state.pool).await;

    // Penjadwal Broadcast (Fase C): lanjutkan campaign 'running' yang tertunda saat boot,
    // lalu tiap 30 detik jalankan campaign terjadwal yang sudah waktunya.
    {
        let sched_state = state.clone();
        tokio::spawn(async move {
            broadcast::resume_running(&sched_state).await;
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                tick.tick().await;
                broadcast::scheduler_tick(&sched_state).await;
            }
        });
    }

    // Auto-downgrade plan kadaluarsa → Basic (saat boot + tiap jam).
    {
        let dg_state = state.clone();
        tokio::spawn(async move {
            billing::downgrade_expired(&dg_state.pool).await;
            billing::send_expiry_reminders(&dg_state.pool).await;
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                tick.tick().await;
                billing::downgrade_expired(&dg_state.pool).await;
                billing::send_expiry_reminders(&dg_state.pool).await;
            }
        });
    }
    let session_layer = SessionManagerLayer::new(store)
        .with_secure(false)
        .with_name("wasync_session");

    // Sub-router User Management — di-gate permission `user.list` (Superadmin bypass).
    let user_routes = Router::new()
        .route("/admin/users", get(users::index).post(users::store))
        .route("/admin/get-datauser", get(users::datatable))
        .route("/admin/users/mass-delete", post(users::mass_delete))
        .route("/admin/users/{id}/edit", get(users::edit))
        .route("/admin/users/{id}/ban", post(users::ban_user))
        .route("/admin/users/{id}/unban", post(users::unban_user))
        .route("/admin/users/{id}", get(users::show).post(users::update).delete(users::destroy))
        .route("/admin/get-user-show-log/{id}", get(users::user_login_sessions))
        .route("/admin/get-user-show-log-activity/{id}", get(users::user_activity))
        .route_layer(from_fn_with_state(state.clone(), rbac::guard_resources));

    // Sub-router Role & Permission — di-gate permission `role.list` (Superadmin bypass).
    let role_routes = Router::new()
        .route("/admin/roles", get(roles::index).post(roles::store))
        .route("/admin/get-datarole", get(roles::datatable))
        .route("/admin/roles/mass-delete", post(roles::mass_delete))
        .route("/admin/roles/generate-permissions", post(roles::generate_permissions))
        .route("/admin/roles/{id}/edit", get(roles::edit))
        .route("/admin/roles/{id}", get(roles::show).post(roles::update).delete(roles::destroy))
        .route("/admin/select/role", get(roles::select))
        .route_layer(from_fn_with_state(state.clone(), rbac::guard_resources));

    // Sub-router Settings + Master Plan — di-gate `view_resources` (hanya admin/Superadmin).
    let settings_routes = Router::new()
        .route("/admin/settings", get(settings::index))
        .route("/admin/settings/update", post(settings::update))
        .route("/admin/plans", get(plans::index))
        .route("/admin/plans/{id}", post(plans::update))
        .route_layer(from_fn_with_state(state.clone(), rbac::guard_resources));

    // Rute yang wajib login.
    let protected = Router::new()
        .route("/admin/dashboard", get(dash::index))
        // --- Notifikasi realtime (WebSocket) ---
        .route("/admin/ws/notifications", get(notify::ws))
        .route("/admin/notify", post(notify::trigger))
        // --- WA Service: Sesi (Fase B P2, nyata) ---
        .route("/admin/wa/sessions", get(wasessions::index).post(wasessions::create))
        .route("/admin/wa/sessions/{id}/qr", get(wasessions::qr))
        .route("/admin/wa/sessions/{id}/restart", post(wasessions::restart))
        .route("/admin/wa/sessions/{id}/level", post(wasessions::set_level))
        .route("/admin/wa/sessions/{id}/webhook", post(wasessions::set_webhook))
        .route("/admin/wa/sessions/{id}/ai", post(wasessions::set_ai))
        .route("/admin/wa/sessions/{id}/simulate", post(wasessions::simulate))
        .route("/admin/wa/sessions/{id}/logout", post(wasessions::logout))
        .route("/admin/wa/sessions/{id}", axum::routing::delete(wasessions::destroy))
        // --- Subscriber management (Superadmin) ---
        .route("/admin/wa/subscribers", get(subscribers::index))
        .route("/admin/wa/subscribers/{id}/plan", post(subscribers::set_plan))
        // --- Broadcast / Campaign (Fase C, nyata) ---
        .route("/admin/wa/broadcast", get(broadcast::index).post(broadcast::create))
        .route("/admin/wa/broadcast/{id}", get(broadcast::show))
        .route("/admin/wa/broadcast/{id}/status", get(broadcast::status))
        .route("/admin/wa/broadcast/{id}/run", post(broadcast::run))
        .route("/admin/wa/broadcast/{id}/cancel", post(broadcast::cancel))
        .route("/admin/wa/templates", post(broadcast::tpl_store))
        .route("/admin/wa/templates/{id}", axum::routing::delete(broadcast::tpl_destroy))
        .route("/admin/wa/messages", get(inbox::index))
        .route("/admin/wa/messages/thread", get(inbox::thread))
        .route("/admin/wa/messages/send", post(inbox::send))
        .route("/admin/wa/messages/resend", post(inbox::resend))
        .route("/admin/wa/messages/contact", post(inbox::set_contact))
        .route("/admin/wa/messages/ai-toggle", post(inbox::ai_toggle))
        // --- Kontak / CRM (Fase Batch 2) ---
        .route("/admin/wa/contacts", get(contacts::index))
        .route("/admin/wa/contacts/save", post(contacts::save))
        // --- Auto-reply keyword (Batch 3) ---
        .route("/admin/wa/autoreply", get(autoreply::index).post(autoreply::create))
        .route("/admin/wa/autoreply/create", post(autoreply::create))
        .route("/admin/wa/autoreply/{id}", axum::routing::delete(autoreply::destroy))
        // --- AI Knowledge Base (Batch 4) ---
        .route("/admin/wa/knowledge", get(knowledge::index).post(knowledge::create))
        .route("/admin/wa/knowledge/create", post(knowledge::create))
        .route("/admin/wa/knowledge/{id}", axum::routing::delete(knowledge::destroy))
        .route("/admin/wa/api-keys", get(wa::api_keys))
        .route("/admin/wa/webhooks", get(wa::webhooks))
        // --- Billing & Plan (Fase P5, Midtrans) ---
        .route("/admin/wa/billing", get(billing::index))
        .route("/admin/wa/billing/checkout", post(billing::checkout))
        .route("/admin/wa/billing/finish", get(billing::finish))
        .route("/admin/wa/billing/invoice/{order_id}", get(billing::invoice))
        // --- Kupon (Superadmin) ---
        .route("/admin/coupons", get(coupons::index).post(coupons::create))
        .route("/admin/coupons/{id}", axum::routing::delete(coupons::destroy))
        // --- User / Role / Settings (gating: view_resources) ---
        .merge(user_routes)
        .merge(role_routes)
        .merge(settings_routes)
        // --- My Account / Profile ---
        .route("/admin/my-account", get(profile::account_index))
        .route("/admin/my-account/{id}/avatar", get(profile::avatar_edit))
        .route("/admin/my-account/{id}/update-avatar", post(profile::avatar_update))
        .route("/admin/my-profile", get(profile::profile_index))
        .route("/admin/my-profile/{id}/edit", get(profile::profile_edit))
        .route("/admin/my-profile/{id}", post(profile::profile_update))
        .route("/admin/my-security", get(profile::security_index).post(profile::security_update))
        .route("/admin/my-security/logout-other-devices", post(profile::logout_other_devices))
        .route("/admin/my-security/{id}/edit", get(profile::security_edit))
        .route("/admin/my-security/{id}", post(profile::security_update))
        .route("/admin/my-activity", get(profile::my_activity_index))
        .route("/admin/mget-my-activity", get(profile::get_my_activity))
        .route("/admin/mmy-login-session", get(profile::my_login_session_index))
        .route("/admin/mget-my-login-session", get(profile::get_my_login_session))
        // --- Help / Log Activity ---
        .route("/admin/log-activity", get(logactivity::index))
        .route("/admin/get-datalogactivity", get(logactivity::datatable))
        .route("/admin/log-activity/{id}", get(logactivity::show))
        // --- Auth lanjutan (protected) ---
        .route("/admin/password", put(authextra::password_update))
        .route("/admin/confirm-password", get(authextra::confirm_show).post(authextra::confirm_store))
        .route("/admin/verify-email", get(authextra::verify_notice))
        .route("/admin/verify-email/{id}/{hash}", get(authextra::verify_link))
        .route("/admin/email/verification-notification", post(authextra::verify_resend))
        .route_layer(from_fn_with_state(state.clone(), maintenance::guard))
        .route_layer(from_fn(auth::require_auth));

    let app = Router::new()
        .route("/", get(landing))
        .route("/health", get(|| async { "ok" }))
        // Endpoint internal dipanggil sidecar WA (di luar require_auth; dijaga secret + loopback)
        .route("/internal/wa/event", post(wainternal::event))
        .route("/internal/wa/usage/{id}", get(wainternal::usage))
        // Public API (di luar require_auth; autentik via app_key + auth_key)
        .route("/api/create-message", post(publicapi::create_message))
        .route("/api/create-message-media", post(publicapi::create_message_media))
        .route("/api/status", get(publicapi::status))
        // Notifikasi pembayaran Midtrans (publik; dijaga signature SHA512, bukan loopback)
        .route("/api/billing/midtrans/notification", post(billing::midtrans_notification))
        .route("/admin/login", get(auth::login_page).post(auth::login_submit))
        .route("/admin/logout", get(auth::logout).post(auth::logout))
        // --- Auth lanjutan (publik/guest) ---
        .route("/admin/register", get(authextra::register_page).post(authextra::register_submit))
        .route("/admin/forgot-password", get(authextra::forgot_page).post(authextra::forgot_submit))
        .route("/admin/reset-password/{token}", get(authextra::reset_page))
        .route("/admin/reset-password", post(authextra::reset_submit))
        // --- Social login (OAuth) ---
        .route("/admin/auth/{provider}/redirect", get(social::redirect))
        .route("/admin/auth/{provider}/callback", get(social::callback))
        .merge(protected)
        .nest_service("/assets", ServeDir::new(assets_dir))
        .nest_service("/storage", ServeDir::new(storage_dir))
        .fallback(not_found)
        .with_state(state)
        .layer(session_layer);

    let addr = "127.0.0.1:8090";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("wa-sync (central/Rust) jalan di http://{addr}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

/// Landing page publik (WA Service) di `/`.
async fn landing(State(state): State<AppState>) -> Html<String> {
    match state.tera.render("landing.html", &tera::Context::new()) {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<pre>Template error: {e:#}</pre>")),
    }
}

/// Fallback 404 — halaman tidak ditemukan.
async fn not_found(State(state): State<AppState>) -> impl IntoResponse {
    let s = settings::all(&state.pool).await;
    let mut ctx = tera::Context::new();
    ctx.insert("site_name", settings::get(&s, "site_name", "StarterTemp"));
    ctx.insert("site_logo", settings::get(&s, "site_logo", "base-logo.png"));
    let html = state
        .tera
        .render("errors/404.html", &ctx)
        .unwrap_or_else(|_| "<h1>404 Not Found</h1>".to_string());
    (StatusCode::NOT_FOUND, Html(html))
}

// (Handler dashboard lama dipindah ke `dash::index` — data nyata per-user.)
