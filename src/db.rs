/// Database abstraction for Postgres (Neon) primary, SQLite fallback for CLI
use std::env;
use std::path::Path;

pub enum DbConnection {
    Postgres,
    Sqlite(rusqlite::Connection),
}

impl DbConnection {
    pub async fn init() -> Result<Self, Box<dyn std::error::Error>> {
        if env::var("DATABASE_URL").is_ok() {
            log::info!("Using Postgres (Neon)");
            Ok(DbConnection::Postgres)
        } else {
            log::info!("Using SQLite fallback");
            let path = "data/sporesec.db";
            if let Some(dir) = Path::new(path).parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let conn = rusqlite::Connection::open(path)?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS clients (id TEXT PRIMARY KEY, paid INTEGER DEFAULT 0)",
                [],
            )?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS token_metadata (jti TEXT PRIMARY KEY, client_id TEXT, exp INTEGER)",
                [],
            )?;
            Ok(DbConnection::Sqlite(conn))
        }
    }

    pub async fn ensure_client_in_db(&self, client_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            DbConnection::Postgres => {
                log::debug!("Postgres client registration: {}", client_id);
                // In production, this would call sqlx
            }
            DbConnection::Sqlite(conn) => {
                conn.execute("INSERT OR IGNORE INTO clients (id, paid) VALUES (?1, 0)", [client_id])?;
            }
        }
        Ok(())
    }

    pub async fn get_client_paid(&self, client_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        match self {
            DbConnection::Postgres => {
                log::debug!("Postgres client paid check: {}", client_id);
                Ok(false) // Placeholder
            }
            DbConnection::Sqlite(conn) => {
                let mut stmt = conn.prepare("SELECT paid FROM clients WHERE id = ?1")?;
                let paid: i32 = stmt.query_row([client_id], |row| row.get(0)).unwrap_or(0);
                Ok(paid == 1)
            }
        }
    }

    pub async fn set_client_paid(&self, client_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            DbConnection::Postgres => {
                log::debug!("Postgres client mark paid: {}", client_id);
            }
            DbConnection::Sqlite(conn) => {
                conn.execute("UPDATE clients SET paid = 1 WHERE id = ?1", [client_id])?;
            }
        }
        Ok(())
    }

    pub async fn persist_token(&self, jti: &str, client_id: &str, exp: i64) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            DbConnection::Postgres => {
                log::debug!("Postgres token persist: {}", jti);
            }
            DbConnection::Sqlite(conn) => {
                conn.execute(
                    "INSERT OR REPLACE INTO token_metadata (jti, client_id, exp) VALUES (?1, ?2, ?3)",
                    rusqlite::params![jti, client_id, exp],
                )?;
            }
        }
        Ok(())
    }

    pub async fn revoke_token(&self, jti: &str) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            DbConnection::Postgres => {
                log::debug!("Postgres token revoke: {}", jti);
            }
            DbConnection::Sqlite(conn) => {
                conn.execute("DELETE FROM token_metadata WHERE jti = ?1", [jti])?;
            }
        }
        Ok(())
    }

    pub async fn is_token_revoked(&self, jti: &str) -> Result<bool, Box<dyn std::error::Error>> {
        match self {
            DbConnection::Postgres => {
                log::debug!("Postgres token revoked check: {}", jti);
                Ok(false) // Placeholder
            }
            DbConnection::Sqlite(conn) => {
                let mut stmt = conn.prepare("SELECT 1 FROM token_metadata WHERE jti = ?1")?;
                let exists = stmt.exists(rusqlite::params![jti])?;
                Ok(!exists)
            }
        }
    }
}
