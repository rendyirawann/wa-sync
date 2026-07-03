//! Seeder idempotent (dijalankan sekali saat boot) — meniru `RolePermissionSeeder` Laravel
//! + menambah role **User**. Membuat permission, role Superadmin/User, dan akun contoh User.

use sqlx::types::Uuid;
use sqlx::PgPool;

/// Permission kanonik aplikasi (selaras RolePermissionSeeder + generate_permissions).
const PERMISSIONS: [&str; 15] = [
    "view_dashboard", "view_data_master", "view_resources", "view_help",
    "user.show", "user.create", "user.edit", "user.delete", "user.massdelete", "user.ban",
    "role.show", "role.create", "role.edit", "role.delete", "role.massdelete",
];

/// Permission untuk role "User": akses dasar saja (TANPA view_resources → tak bisa kelola user/role/settings).
const USER_PERMS: [&str; 2] = ["view_dashboard", "view_help"];

pub async fn run(pool: &PgPool) {
    // 1) Permissions (idempotent)
    for name in PERMISSIONS {
        if let Some((prefix, _)) = name.split_once('.') {
            let category = title_case(prefix);
            let _ = sqlx::query(
                "INSERT INTO permissions (name, guard_name, category, created_at, updated_at) \
                 VALUES ($1,'web',$2,now(),now()) ON CONFLICT (name, guard_name) DO NOTHING",
            ).bind(name).bind(&category).execute(pool).await;
        } else {
            let _ = sqlx::query(
                "INSERT INTO permissions (name, guard_name, created_at, updated_at) \
                 VALUES ($1,'web',now(),now()) ON CONFLICT (name, guard_name) DO NOTHING",
            ).bind(name).execute(pool).await;
        }
    }

    // 2) Roles
    let superadmin_id = ensure_role(pool, "Superadmin").await;
    let user_id = ensure_role(pool, "User").await;

    // 3) Assign permission ke role
    if let Some(sid) = superadmin_id {
        // Superadmin: semua permission (web)
        let _ = sqlx::query(
            "INSERT INTO role_has_permissions (permission_id, role_id) \
             SELECT p.id, $1 FROM permissions p WHERE p.guard_name = 'web' \
             ON CONFLICT DO NOTHING",
        ).bind(sid).execute(pool).await;
    }
    if let Some(uid) = user_id {
        for p in USER_PERMS {
            let _ = sqlx::query(
                "INSERT INTO role_has_permissions (permission_id, role_id) \
                 SELECT p.id, $1 FROM permissions p WHERE p.name = $2 AND p.guard_name = 'web' \
                 ON CONFLICT DO NOTHING",
            ).bind(uid).bind(p).execute(pool).await;
        }
        // 4) Akun contoh dengan role User (untuk menguji pembedaan menu/dashboard)
        ensure_sample_user(pool, uid).await;
    }
}

async fn ensure_role(pool: &PgPool, name: &str) -> Option<i64> {
    let _ = sqlx::query(
        "INSERT INTO roles (name, guard_name, created_at, updated_at) \
         VALUES ($1,'web',now(),now()) ON CONFLICT (name, guard_name) DO NOTHING",
    ).bind(name).execute(pool).await;
    sqlx::query_scalar("SELECT id FROM roles WHERE name = $1 AND guard_name = 'web'")
        .bind(name).fetch_optional(pool).await.ok().flatten()
}

async fn ensure_sample_user(pool: &PgPool, user_role_id: i64) {
    let email = "user@gmail.com";
    let existing: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(email).fetch_optional(pool).await.ok().flatten();
    let uid = match existing {
        Some(id) => id,
        None => {
            let id = Uuid::now_v7();
            let hash = bcrypt::hash("user12345", 12).unwrap_or_default();
            let username = format!("user-{}", &id.to_string()[..8]);
            let res = sqlx::query(
                "INSERT INTO users (id, name, username, email, password, is_active, email_verified_at, created_at, updated_at) \
                 VALUES ($1,'User Demo',$2,$3,$4,true,now(),now(),now())",
            ).bind(id).bind(&username).bind(email).bind(&hash).execute(pool).await;
            if res.is_err() {
                return;
            }
            tracing::info!("Seed: akun contoh dibuat — user@gmail.com / user12345 (role User)");
            id
        }
    };
    let _ = sqlx::query(
        "INSERT INTO model_has_roles (role_id, model_type, model_id) \
         VALUES ($1, 'App\\Models\\User', $2) ON CONFLICT DO NOTHING",
    ).bind(user_role_id).bind(uid).execute(pool).await;
}

fn title_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}
