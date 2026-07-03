//! Session store persisten berbasis Postgres (tabel `sessions`), memakai pool sqlx 0.9
//! yang sama dengan aplikasi. Menggantikan `MemoryStore` agar sesi login bertahan
//! saat server di-restart (tidak ter-logout setiap reload).

use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use sqlx::PgPool;
use tower_sessions::session::{Id, Record};
use tower_sessions::session_store::{Error, Result, SessionStore};

#[derive(Clone, Debug)]
pub struct PgStore {
    pool: PgPool,
}

impl PgStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Buat tabel sesi bila belum ada (idempotent, dipanggil sekali saat boot).
    pub async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS rust_sessions (\
                 id TEXT PRIMARY KEY, \
                 data TEXT NOT NULL, \
                 expiry BIGINT NOT NULL\
             )",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[async_trait]
impl SessionStore for PgStore {
    async fn create(&self, record: &mut Record) -> Result<()> {
        // Id berupa i128 acak → tabrakan praktis nihil; cukup simpan seperti save.
        self.save(record).await
    }

    async fn save(&self, record: &Record) -> Result<()> {
        let id = record.id.to_string();
        let data = serde_json::to_string(record).map_err(|e| Error::Encode(e.to_string()))?;
        let expiry = record.expiry_date.unix_timestamp();
        sqlx::query(
            "INSERT INTO rust_sessions (id, data, expiry) VALUES ($1, $2, $3) \
             ON CONFLICT (id) DO UPDATE SET data = EXCLUDED.data, expiry = EXCLUDED.expiry",
        )
        .bind(&id)
        .bind(&data)
        .bind(expiry)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Backend(e.to_string()))?;
        Ok(())
    }

    async fn load(&self, session_id: &Id) -> Result<Option<Record>> {
        let id = session_id.to_string();
        let row: Option<(String,)> =
            sqlx::query_as("SELECT data FROM rust_sessions WHERE id = $1 AND expiry > $2")
                .bind(&id)
                .bind(now_unix())
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| Error::Backend(e.to_string()))?;
        match row {
            Some((data,)) => {
                let rec: Record =
                    serde_json::from_str(&data).map_err(|e| Error::Decode(e.to_string()))?;
                Ok(Some(rec))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, session_id: &Id) -> Result<()> {
        let id = session_id.to_string();
        sqlx::query("DELETE FROM rust_sessions WHERE id = $1")
            .bind(&id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Backend(e.to_string()))?;
        Ok(())
    }
}
