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

/// Seed akun SUBSCRIBER Enterprise (role User, langganan 1 tahun) + data contoh nyata
/// (sesi, kontak, pesan, pemakaian) supaya dashboard subscriber terisi data asli dari DB.
/// Dipanggil SETELAH wamigrate (butuh tabel plans + kolom plan_id + wa_*).
/// Idempotent: hanya membuat bila akun belum ada.
pub async fn enterprise_subscriber(pool: &PgPool) {
    let email = "enterprise@gmail.com";
    let exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email=$1")
        .bind(email).fetch_optional(pool).await.ok().flatten();
    if exists.is_some() {
        return; // sudah ada → biarkan (dikelola via halaman Subscriber)
    }
    let role_id: Option<i64> = sqlx::query_scalar("SELECT id FROM roles WHERE name='User' AND guard_name='web'")
        .fetch_optional(pool).await.ok().flatten();
    let plan_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM plans WHERE code='enterprise'")
        .fetch_optional(pool).await.ok().flatten();
    let (Some(role_id), Some(plan_id)) = (role_id, plan_id) else { return };

    let uid = Uuid::now_v7();
    let hash = bcrypt::hash("enterprise12345", 12).unwrap_or_default();
    let username = format!("enterprise-{}", &uid.to_string()[..8]);
    let res = sqlx::query(
        "INSERT INTO users (id,name,username,email,password,is_active,email_verified_at,plan_id,plan_started_at,plan_expires_at,created_at,updated_at) \
         VALUES ($1,'Rendy Irawan (Enterprise)',$2,$3,$4,true,now(),$5,now(),now()+interval '1 year',now(),now())",
    ).bind(uid).bind(&username).bind(email).bind(&hash).bind(plan_id).execute(pool).await;
    if res.is_err() {
        return;
    }
    let _ = sqlx::query("INSERT INTO model_has_roles (role_id,model_type,model_id) VALUES ($1,'App\\Models\\User',$2) ON CONFLICT DO NOTHING")
        .bind(role_id).bind(uid).execute(pool).await;

    // --- data contoh (nyata di DB, agar dashboard/inbox/broadcast subscriber terisi) ---
    let s1 = Uuid::now_v7();
    let s2 = Uuid::now_v7();
    for (sid, label, phone) in [(s1, "CS Toko Online", "628111000001"), (s2, "Marketing HustleSync", "628111000002")] {
        let ak = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let ws = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let _ = sqlx::query(
            "INSERT INTO wa_sessions (id,user_id,label,phone,jid,status,antiban_level,sim_mode,auth_key,webhook_secret,last_connected_at,created_at,updated_at) \
             VALUES ($1,$2,$3,$4,$5,'connected','normal',true,$6,$7,now(),now(),now())",
        ).bind(sid).bind(uid).bind(label).bind(phone).bind(format!("{phone}@s.whatsapp.net")).bind(&ak).bind(&ws).execute(pool).await;
    }
    for (sid, jid, name) in [
        (s1, "628222000011@s.whatsapp.net", "Budi Santoso"),
        (s1, "628222000012@s.whatsapp.net", "Sari Dewi"),
        (s2, "628222000021@s.whatsapp.net", "Toko Berkah"),
    ] {
        let _ = sqlx::query("INSERT INTO wa_contacts (session_id,jid,name) VALUES ($1,$2,$3) ON CONFLICT DO NOTHING")
            .bind(sid).bind(jid).bind(name).execute(pool).await;
    }
    let convos: [(Uuid, &str); 3] = [(s1, "628222000011"), (s1, "628222000012"), (s2, "628222000021")];
    for d in 0..10i32 {
        for (sid, ph) in convos {
            let jid = format!("{ph}@s.whatsapp.net");
            let _ = sqlx::query(
                "INSERT INTO wa_messages (id,session_id,direction,remote_jid,body,msg_type,status,created_at) \
                 VALUES ($1,$2,'in',$3,'Halo kak, produknya masih ready?','text','received',now()-make_interval(days=>$4::int))",
            ).bind(Uuid::now_v7()).bind(sid).bind(&jid).bind(d).execute(pool).await;
            let _ = sqlx::query(
                "INSERT INTO wa_messages (id,session_id,direction,remote_jid,body,msg_type,status,via_ai,created_at) \
                 VALUES ($1,$2,'out',$3,'Ready kak, silakan order ya. Ada yang bisa dibantu lagi?','text','sent',true,now()-make_interval(days=>$4::int))",
            ).bind(Uuid::now_v7()).bind(sid).bind(&jid).bind(d).execute(pool).await;
        }
    }
    for d in 0..10i32 {
        for (sid, sent, recv) in [(s1, 2i32, 2i32), (s2, 1i32, 1i32)] {
            let _ = sqlx::query(
                "INSERT INTO wa_usage_daily (session_id,day,sent_count,received_count,updated_at) \
                 VALUES ($1,(CURRENT_DATE - $2::int),$3,$4,now()) ON CONFLICT (session_id,day) DO NOTHING",
            ).bind(sid).bind(d).bind(sent).bind(recv).execute(pool).await;
        }
    }
    tracing::info!("Seed: akun Enterprise — enterprise@gmail.com / enterprise12345 (role User, langganan 1thn) + data contoh");
}

fn title_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}
