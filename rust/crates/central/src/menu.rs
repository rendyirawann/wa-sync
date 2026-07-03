//! Menu sidebar DINAMIS dari tabel `menus` (idempotent saat boot + seed default).
//! Di-render oleh shell `wa.html` & `admin.html` via `menu_groups` di base_context.
//! Gating per-item lewat kolom `permission` (null = semua; Superadmin bypass via can()).

use serde::Serialize;
use sqlx::PgPool;

use crate::rbac::CurrentUser;

#[derive(Serialize)]
pub struct MenuItem {
    pub label: String,
    pub icon: String,
    pub url: String,
    pub active_key: String,
    pub badge: Option<String>,
}

#[derive(Serialize)]
pub struct MenuGroup {
    pub section: String,
    pub items: Vec<MenuItem>,
}

const DDL: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS menus (\
        id uuid PRIMARY KEY DEFAULT gen_random_uuid(), \
        section text NOT NULL DEFAULT 'Menu', \
        label text NOT NULL, \
        icon text NOT NULL DEFAULT 'grid', \
        url text NOT NULL DEFAULT '#', \
        active_key text NOT NULL DEFAULT '', \
        permission text, \
        badge text, \
        sort_order int NOT NULL DEFAULT 0, \
        enabled boolean NOT NULL DEFAULT true, \
        created_at timestamptz NOT NULL DEFAULT now(), \
        updated_at timestamptz NOT NULL DEFAULT now())",
    "CREATE INDEX IF NOT EXISTS idx_menus_sort ON menus (sort_order)",
];

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    for &sql in DDL {
        sqlx::query(sql).execute(pool).await?;
    }
    Ok(())
}

/// Isi menu default bila tabel masih kosong (tidak menimpa kustomisasi user).
pub async fn seed(pool: &PgPool) {
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM menus").fetch_one(pool).await.unwrap_or(0);
    if count > 0 {
        return;
    }
    // (section, label, icon, url, active_key, permission, sort)
    let items: &[(&str, &str, &str, &str, &str, Option<&str>, i32)] = &[
        ("Menu", "Dashboard", "grid", "/admin/dashboard", "dashboard", None, 10),
        ("Menu", "Sesi WhatsApp", "device", "/admin/wa/sessions", "sessions", None, 20),
        ("Menu", "Subscriber", "users", "/admin/wa/subscribers", "subscribers", None, 30),
        ("Menu", "Broadcast", "cast", "/admin/wa/broadcast", "broadcast", None, 40),
        ("Menu", "Pesan", "chat", "/admin/wa/messages", "messages", None, 50),
        ("Developer", "API Keys", "key", "/admin/wa/api-keys", "api-keys", None, 60),
        ("Developer", "Webhooks", "plug", "/admin/wa/webhooks", "webhooks", None, 70),
        ("Developer", "Log Aktivitas", "list", "/admin/log-activity", "log-activity", None, 80),
        ("Akun", "Billing & Plan", "card", "/admin/wa/billing", "billing", None, 90),
        ("Sistem", "User Management", "user", "/admin/users", "users", Some("view_resources"), 100),
        ("Sistem", "Roles & Permission", "shield", "/admin/roles", "roles", Some("view_resources"), 110),
        ("Sistem", "Master Plan", "card", "/admin/plans", "plans", Some("view_resources"), 115),
        ("Sistem", "Pengaturan", "gear", "/admin/settings", "settings", Some("view_resources"), 120),
    ];
    for &(section, label, icon, url, active_key, permission, sort) in items {
        let _ = sqlx::query(
            "INSERT INTO menus (section,label,icon,url,active_key,permission,sort_order) VALUES ($1,$2,$3,$4,$5,$6,$7)",
        )
        .bind(section).bind(label).bind(icon).bind(url).bind(active_key).bind(permission).bind(sort)
        .execute(pool).await;
    }
    tracing::info!("Seed menu default: {} item", items.len());
}

/// Sisipkan item menu yang ditambahkan SETELAH seed awal (idempotent per-url).
/// Dipanggil tiap boot agar DB lama ikut dapat menu baru tanpa menimpa kustomisasi.
pub async fn ensure(pool: &PgPool) {
    // (section, label, icon, url, active_key, permission, sort)
    let extras: &[(&str, &str, &str, &str, &str, Option<&str>, i32)] = &[
        ("Sistem", "Master Plan", "card", "/admin/plans", "plans", Some("view_resources"), 115),
        ("Menu", "Kontak", "users", "/admin/wa/contacts", "contacts", None, 55),
    ];
    for &(section, label, icon, url, active_key, permission, sort) in extras {
        let _ = sqlx::query(
            "INSERT INTO menus (section,label,icon,url,active_key,permission,sort_order) \
             SELECT $1,$2,$3,$4,$5,$6,$7 WHERE NOT EXISTS (SELECT 1 FROM menus WHERE url=$4)",
        )
        .bind(section).bind(label).bind(icon).bind(url).bind(active_key).bind(permission).bind(sort)
        .execute(pool).await;
    }
    // Gate menu khusus Superadmin/admin (butuh view_resources). Subscriber mgmt = alat admin.
    let _ = sqlx::query("UPDATE menus SET permission='view_resources' WHERE url='/admin/wa/subscribers' AND (permission IS NULL OR permission='')")
        .execute(pool).await;
}

/// Muat menu aktif, filter per-izin user, kelompokkan per-section (urut sort_order).
pub async fn load(pool: &PgPool, user: &CurrentUser) -> Vec<MenuGroup> {
    let rows: Vec<(String, String, String, String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT section, label, icon, url, active_key, permission, badge FROM menus WHERE enabled ORDER BY sort_order",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut groups: Vec<MenuGroup> = Vec::new();
    for (section, label, icon, url, active_key, permission, badge) in rows {
        // gating per-item: permission kosong/null = semua; selain itu cek can() (Superadmin bypass)
        if let Some(p) = permission.as_deref() {
            if !p.is_empty() && !user.can(p) {
                continue;
            }
        }
        let item = MenuItem { label, icon, url, active_key, badge };
        match groups.last_mut() {
            Some(g) if g.section == section => g.items.push(item),
            _ => groups.push(MenuGroup { section, items: vec![item] }),
        }
    }
    groups
}
