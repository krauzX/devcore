use crate::error::DevCoreError;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

const CORE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS kv_store (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    pub fn open(project_root: &Path) -> Result<Self, DevCoreError> {
        let db_dir = project_root.join(".devcore");
        std::fs::create_dir_all(&db_dir)?;
        let db_path = db_dir.join("devcore.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(CORE_SCHEMA)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, DevCoreError> {
        self.conn
            .lock()
            .map_err(|e| DevCoreError::Config(format!("Lock poisoned: {}", e)))
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, DevCoreError> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT value FROM kv_store WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(Ok(val)) => Ok(Some(val)),
            _ => Ok(None),
        }
    }

    pub fn set(&self, key: &str, value: &str) -> Result<(), DevCoreError> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO kv_store (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<(), DevCoreError> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM kv_store WHERE key = ?1", params![key])?;
        Ok(())
    }
}
