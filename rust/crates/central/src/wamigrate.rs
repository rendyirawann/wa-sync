//! Migrasi tabel WhatsApp (Fase B) — idempotent, dijalankan sekali saat boot
//! (pola sama seperti `seed.rs`/`sessionstore.rs`). Semua tabel ber-prefiks `wa_`.
//! `gen_random_uuid()` = bawaan PostgreSQL 13+ (di sini hanya jaring pengaman;
//! id sebenarnya dibuat di Rust via `Uuid::now_v7()`).

use sqlx::PgPool;

const STMTS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS wa_sessions (\
        id uuid PRIMARY KEY DEFAULT gen_random_uuid(), \
        user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE, \
        label text NOT NULL DEFAULT '', \
        phone text, jid text, \
        status text NOT NULL DEFAULT 'pending' \
            CHECK (status IN ('pending','qr','connecting','connected','disconnected','banned','logged_out')), \
        qr text, last_connected_at timestamptz, \
        meta jsonb NOT NULL DEFAULT '{}'::jsonb, \
        created_at timestamptz NOT NULL DEFAULT now(), \
        updated_at timestamptz NOT NULL DEFAULT now())",
    "CREATE INDEX IF NOT EXISTS idx_wa_sessions_user_id ON wa_sessions (user_id)",
    "CREATE INDEX IF NOT EXISTS idx_wa_sessions_status ON wa_sessions (status)",
    "CREATE UNIQUE INDEX IF NOT EXISTS uq_wa_sessions_user_name ON wa_sessions (user_id, label) WHERE label <> ''",
    "CREATE TABLE IF NOT EXISTS wa_messages (\
        id uuid PRIMARY KEY DEFAULT gen_random_uuid(), \
        session_id uuid NOT NULL REFERENCES wa_sessions(id) ON DELETE CASCADE, \
        direction text NOT NULL CHECK (direction IN ('in','out')), \
        remote_jid text NOT NULL, body text, \
        msg_type text NOT NULL DEFAULT 'text', \
        wa_message_id text, \
        status text NOT NULL DEFAULT 'pending' \
            CHECK (status IN ('pending','sent','delivered','read','failed','received')), \
        error text, \
        created_at timestamptz NOT NULL DEFAULT now())",
    "CREATE INDEX IF NOT EXISTS idx_wa_messages_session_id ON wa_messages (session_id)",
    "CREATE INDEX IF NOT EXISTS idx_wa_messages_remote_jid ON wa_messages (session_id, remote_jid)",
    "CREATE UNIQUE INDEX IF NOT EXISTS uq_wa_messages_session_waid ON wa_messages (session_id, wa_message_id) WHERE wa_message_id IS NOT NULL",

    // ===== Fase B (SaaS): plans, kredensial, kuota, webhook, antiban, AI =====
    "CREATE TABLE IF NOT EXISTS plans (\
        id uuid PRIMARY KEY DEFAULT gen_random_uuid(), \
        code text NOT NULL UNIQUE CHECK (code IN ('basic','premium','enterprise')), \
        name text NOT NULL, \
        max_sessions int NOT NULL DEFAULT 2, \
        max_messages_per_day int NOT NULL DEFAULT 0, \
        price_idr bigint NOT NULL DEFAULT 0, \
        ai_enabled boolean NOT NULL DEFAULT false, \
        webhook_enabled boolean NOT NULL DEFAULT false, \
        is_active boolean NOT NULL DEFAULT true, \
        sort_order int NOT NULL DEFAULT 0, \
        created_at timestamptz NOT NULL DEFAULT now(), \
        updated_at timestamptz NOT NULL DEFAULT now())",
    "INSERT INTO plans (code,name,max_sessions,max_messages_per_day,price_idr,ai_enabled,webhook_enabled,sort_order) VALUES \
        ('basic','Basic',2,200,0,false,false,1), \
        ('premium','Premium',5,1000,99000,true,true,2), \
        ('enterprise','Enterprise',10,5000,299000,true,true,3) \
        ON CONFLICT (code) DO UPDATE SET max_sessions=EXCLUDED.max_sessions, max_messages_per_day=EXCLUDED.max_messages_per_day, ai_enabled=EXCLUDED.ai_enabled, webhook_enabled=EXCLUDED.webhook_enabled",
    // app_key = per USER (sama untuk semua nomor user); auth_key = per NOMOR (di wa_sessions)
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS plan_id uuid REFERENCES plans(id) ON DELETE SET NULL",
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS plan_started_at timestamptz",
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS plan_expires_at timestamptz",
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS app_key uuid NOT NULL DEFAULT gen_random_uuid()",
    "CREATE UNIQUE INDEX IF NOT EXISTS uq_users_app_key ON users (app_key)",
    "CREATE INDEX IF NOT EXISTS idx_users_plan_id ON users (plan_id)",
    "UPDATE users SET plan_id=(SELECT id FROM plans WHERE code='basic') WHERE plan_id IS NULL",
    // wa_sessions: auth_key per-nomor + webhook + antiban + AI config
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS auth_key text",
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS auth_key_rotated_at timestamptz NOT NULL DEFAULT now()",
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS webhook_url text",
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS webhook_secret text",
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS webhook_enabled boolean NOT NULL DEFAULT false",
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS webhook_events text[] NOT NULL DEFAULT ARRAY['message','sent','connected','disconnected']::text[]",
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS antiban_level text NOT NULL DEFAULT 'normal' CHECK (antiban_level IN ('safe','normal','aggressive'))",
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS sim_mode boolean NOT NULL DEFAULT false",
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS ai_enabled boolean NOT NULL DEFAULT false",
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS ai_model text NOT NULL DEFAULT 'qwen2.5'",
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS ai_system_prompt text NOT NULL DEFAULT ''",
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS ai_history_limit int NOT NULL DEFAULT 10",
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS ai_only_when_idle_min int NOT NULL DEFAULT 0",
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS ai_reply_groups boolean NOT NULL DEFAULT false",
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS ai_hours_start smallint NOT NULL DEFAULT 0",
    "ALTER TABLE wa_sessions ADD COLUMN IF NOT EXISTS ai_hours_end smallint NOT NULL DEFAULT 24",
    "ALTER TABLE wa_messages ADD COLUMN IF NOT EXISTS via_ai boolean NOT NULL DEFAULT false",
    "CREATE TABLE IF NOT EXISTS wa_usage_daily (\
        session_id uuid NOT NULL REFERENCES wa_sessions(id) ON DELETE CASCADE, \
        day date NOT NULL DEFAULT CURRENT_DATE, \
        sent_count int NOT NULL DEFAULT 0, \
        received_count int NOT NULL DEFAULT 0, \
        ai_reply_count int NOT NULL DEFAULT 0, \
        failed_count int NOT NULL DEFAULT 0, \
        updated_at timestamptz NOT NULL DEFAULT now(), \
        PRIMARY KEY (session_id, day))",
    "CREATE INDEX IF NOT EXISTS idx_wa_usage_daily_day ON wa_usage_daily (day)",
    "CREATE TABLE IF NOT EXISTS wa_webhook_deliveries (\
        id uuid PRIMARY KEY DEFAULT gen_random_uuid(), \
        session_id uuid NOT NULL REFERENCES wa_sessions(id) ON DELETE CASCADE, \
        event text NOT NULL, url text NOT NULL, status_code int, \
        attempts int NOT NULL DEFAULT 0, last_error text, \
        delivered boolean NOT NULL DEFAULT false, \
        created_at timestamptz NOT NULL DEFAULT now())",
    "CREATE INDEX IF NOT EXISTS idx_wa_webhook_deliveries_session ON wa_webhook_deliveries (session_id, created_at DESC)",
    // ===== Fase C: Broadcast (campaign + target + template) =====
    "CREATE TABLE IF NOT EXISTS wa_campaigns (\
        id uuid PRIMARY KEY DEFAULT gen_random_uuid(), \
        user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE, \
        session_id uuid NOT NULL REFERENCES wa_sessions(id) ON DELETE CASCADE, \
        name text NOT NULL DEFAULT '', \
        body text NOT NULL DEFAULT '', \
        status text NOT NULL DEFAULT 'draft' \
            CHECK (status IN ('draft','scheduled','running','paused','done','canceled')), \
        scheduled_at timestamptz, \
        total int NOT NULL DEFAULT 0, \
        sent_count int NOT NULL DEFAULT 0, \
        failed_count int NOT NULL DEFAULT 0, \
        last_error text, \
        created_at timestamptz NOT NULL DEFAULT now(), \
        started_at timestamptz, \
        finished_at timestamptz, \
        updated_at timestamptz NOT NULL DEFAULT now())",
    "CREATE INDEX IF NOT EXISTS idx_wa_campaigns_user ON wa_campaigns (user_id, created_at DESC)",
    "CREATE INDEX IF NOT EXISTS idx_wa_campaigns_session ON wa_campaigns (session_id)",
    "CREATE INDEX IF NOT EXISTS idx_wa_campaigns_due ON wa_campaigns (status, scheduled_at) WHERE status='scheduled'",
    "CREATE TABLE IF NOT EXISTS wa_campaign_targets (\
        id uuid PRIMARY KEY DEFAULT gen_random_uuid(), \
        campaign_id uuid NOT NULL REFERENCES wa_campaigns(id) ON DELETE CASCADE, \
        phone text NOT NULL, \
        name text, \
        vars jsonb NOT NULL DEFAULT '{}'::jsonb, \
        status text NOT NULL DEFAULT 'pending' \
            CHECK (status IN ('pending','sent','failed','skipped')), \
        rendered_body text, \
        wa_message_id text, \
        error text, \
        sent_at timestamptz)",
    "CREATE INDEX IF NOT EXISTS idx_wa_campaign_targets_camp ON wa_campaign_targets (campaign_id, status)",
    "CREATE UNIQUE INDEX IF NOT EXISTS uq_wa_campaign_targets_phone ON wa_campaign_targets (campaign_id, phone)",
    "CREATE TABLE IF NOT EXISTS wa_message_templates (\
        id uuid PRIMARY KEY DEFAULT gen_random_uuid(), \
        user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE, \
        name text NOT NULL, \
        body text NOT NULL DEFAULT '', \
        created_at timestamptz NOT NULL DEFAULT now(), \
        updated_at timestamptz NOT NULL DEFAULT now())",
    "CREATE INDEX IF NOT EXISTS idx_wa_message_templates_user ON wa_message_templates (user_id, created_at DESC)",
    // ===== Fase P5: Billing (invoice + pembayaran Midtrans Snap) =====
    "CREATE TABLE IF NOT EXISTS wa_invoices (\
        id uuid PRIMARY KEY DEFAULT gen_random_uuid(), \
        order_id text NOT NULL UNIQUE, \
        user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE, \
        plan_id uuid NOT NULL REFERENCES plans(id), \
        plan_code text NOT NULL DEFAULT '', \
        plan_name text NOT NULL DEFAULT '', \
        amount_idr bigint NOT NULL, \
        period_days int NOT NULL DEFAULT 30, \
        status text NOT NULL DEFAULT 'pending' \
            CHECK (status IN ('pending','paid','failed','expired','canceled')), \
        snap_token text, \
        snap_redirect_url text, \
        midtrans_transaction_status text, \
        midtrans_payment_type text, \
        paid_at timestamptz, \
        applied_at timestamptz, \
        raw_notification jsonb, \
        created_at timestamptz NOT NULL DEFAULT now(), \
        updated_at timestamptz NOT NULL DEFAULT now())",
    "CREATE INDEX IF NOT EXISTS idx_wa_invoices_user ON wa_invoices (user_id, created_at DESC)",
    "CREATE INDEX IF NOT EXISTS idx_wa_invoices_status ON wa_invoices (status)",
    // Nama kontak (alias) per nomor — mis. simpan +62812... sebagai "Suci".
    "CREATE TABLE IF NOT EXISTS wa_contacts (\
        session_id uuid NOT NULL REFERENCES wa_sessions(id) ON DELETE CASCADE, \
        jid text NOT NULL, \
        name text NOT NULL DEFAULT '', \
        updated_at timestamptz NOT NULL DEFAULT now(), \
        PRIMARY KEY (session_id, jid))",
    // Persona AI khusus per-kontak (override system prompt sesi). Mis. "balas seperti pacar".
    "ALTER TABLE wa_contacts ADD COLUMN IF NOT EXISTS ai_persona text NOT NULL DEFAULT ''",
    // Batch 2: CRM (tag & catatan kontak) + media pada pesan.
    "ALTER TABLE wa_contacts ADD COLUMN IF NOT EXISTS tags text NOT NULL DEFAULT ''",
    "ALTER TABLE wa_contacts ADD COLUMN IF NOT EXISTS notes text NOT NULL DEFAULT ''",
    "ALTER TABLE wa_messages ADD COLUMN IF NOT EXISTS media_url text",
    "ALTER TABLE wa_messages ADD COLUMN IF NOT EXISTS media_mime text",
    "ALTER TABLE wa_messages ADD COLUMN IF NOT EXISTS media_name text",
    // Batch 3: auto-reply keyword (rule-based) + human handoff (pause AI per-kontak).
    "CREATE TABLE IF NOT EXISTS wa_autoreply (\
        id uuid PRIMARY KEY DEFAULT gen_random_uuid(), \
        session_id uuid NOT NULL REFERENCES wa_sessions(id) ON DELETE CASCADE, \
        keyword text NOT NULL, \
        match_type text NOT NULL DEFAULT 'contains' CHECK (match_type IN ('contains','exact','starts')), \
        reply text NOT NULL DEFAULT '', \
        enabled boolean NOT NULL DEFAULT true, \
        sort_order int NOT NULL DEFAULT 0, \
        created_at timestamptz NOT NULL DEFAULT now())",
    "CREATE INDEX IF NOT EXISTS idx_wa_autoreply_session ON wa_autoreply (session_id)",
    "ALTER TABLE wa_contacts ADD COLUMN IF NOT EXISTS ai_paused boolean NOT NULL DEFAULT false",
    // Batch 4: knowledge base AI (FAQ/katalog) — disuntik ke prompt saat relevan (RAG-lite).
    "CREATE TABLE IF NOT EXISTS wa_knowledge (\
        id uuid PRIMARY KEY DEFAULT gen_random_uuid(), \
        session_id uuid NOT NULL REFERENCES wa_sessions(id) ON DELETE CASCADE, \
        title text NOT NULL DEFAULT '', \
        content text NOT NULL DEFAULT '', \
        enabled boolean NOT NULL DEFAULT true, \
        created_at timestamptz NOT NULL DEFAULT now())",
    "CREATE INDEX IF NOT EXISTS idx_wa_knowledge_session ON wa_knowledge (session_id)",
    // Batch 5: billing lanjutan (reminder expiry + kupon + invoice).
    "ALTER TABLE users ADD COLUMN IF NOT EXISTS plan_reminded_for timestamptz",
    "CREATE TABLE IF NOT EXISTS wa_coupons (\
        id uuid PRIMARY KEY DEFAULT gen_random_uuid(), \
        code text NOT NULL UNIQUE, \
        percent int NOT NULL DEFAULT 0 CHECK (percent BETWEEN 0 AND 100), \
        active boolean NOT NULL DEFAULT true, \
        expires_at timestamptz, \
        max_uses int NOT NULL DEFAULT 0, \
        used_count int NOT NULL DEFAULT 0, \
        created_at timestamptz NOT NULL DEFAULT now())",
    "ALTER TABLE wa_invoices ADD COLUMN IF NOT EXISTS coupon_code text",
];

pub async fn run(pool: &PgPool) -> anyhow::Result<()> {
    for &sql in STMTS {
        sqlx::query(sql).execute(pool).await?;
    }
    tracing::info!("Migrasi tabel WA siap (wa_sessions, wa_messages)");
    Ok(())
}
