use crate::error::DevCoreError;
use crate::models::*;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS change_receipts (
    id TEXT PRIMARY KEY, commit_oid TEXT NOT NULL, timestamp TEXT NOT NULL,
    is_ai_generated INTEGER NOT NULL DEFAULT 0, ai_source TEXT,
    intent TEXT NOT NULL DEFAULT '', risk_score INTEGER NOT NULL DEFAULT 0,
    receipt_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS workflow_events (
    id TEXT PRIMARY KEY, timestamp TEXT NOT NULL,
    event_type TEXT NOT NULL, details_json TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_receipts_commit ON change_receipts(commit_oid);
CREATE INDEX IF NOT EXISTS idx_receipts_timestamp ON change_receipts(timestamp);
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON workflow_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_events_type ON workflow_events(event_type);
"#;

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

type Result<T> = std::result::Result<T, DevCoreError>;

/// Persistent storage backed by SQLite for change receipts and workflow events.
pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    pub fn open(project_root: &Path) -> Result<Self> {
        let db_dir = project_root.join(".devcore");
        std::fs::create_dir_all(&db_dir)?;
        let db_path = db_dir.join("devcore.db");
        let conn = Connection::open(&db_path).map_err(|e| {
            DevCoreError::Config(format!("Failed to open {}: {}", db_path.display(), e))
        })?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|e| DevCoreError::Io(std::io::Error::other(format!("Lock poisoned: {}", e))))
    }

    fn query_receipts(
        &self,
        sql: &str,
        params: impl rusqlite::Params,
    ) -> Result<Vec<ChangeReceipt>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params, |row| row.get::<_, String>(0))?;
        rows.filter_map(|r| r.ok())
            .map(|j| serde_json::from_str(&j))
            .collect::<std::result::Result<_, _>>()
            .map_err(Into::into)
    }

    fn query_events(&self, sql: &str, params: impl rusqlite::Params) -> Result<Vec<WorkflowEvent>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params, |row| row.get::<_, String>(0))?;
        rows.filter_map(|r| r.ok())
            .map(|j| serde_json::from_str(&j))
            .collect::<std::result::Result<_, _>>()
            .map_err(Into::into)
    }

    /// Persists a change receipt, replacing any existing receipt with the same ID.
    pub fn save_receipt(&self, receipt: &ChangeReceipt) -> Result<()> {
        let conn = self.lock()?;
        let json = serde_json::to_string(receipt)?;
        conn.execute(
            "INSERT OR REPLACE INTO change_receipts (id, commit_oid, timestamp, is_ai_generated, ai_source, intent, risk_score, receipt_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![receipt.id, receipt.commit_oid, receipt.timestamp.to_rfc3339(), receipt.is_ai_generated as i32, receipt.ai_source.as_ref().map(|s| format!("{:?}", s)), receipt.intent, receipt.risk_score, json],
        )?;
        Ok(())
    }

    /// Retrieves a change receipt by commit OID. Returns `None` if not found.
    pub fn get_receipt(&self, commit_oid: &str) -> Result<Option<ChangeReceipt>> {
        self.query_receipts(
            "SELECT receipt_json FROM change_receipts WHERE commit_oid = ?1",
            params![commit_oid],
        )
        .map(|mut v| v.pop())
    }

    pub fn recent_receipts(&self, limit: usize) -> Result<Vec<ChangeReceipt>> {
        self.query_receipts(
            "SELECT receipt_json FROM change_receipts ORDER BY timestamp DESC LIMIT ?1",
            params![limit as i64],
        )
    }

    pub fn receipts_for_file(&self, file_path: &str) -> Result<Vec<ChangeReceipt>> {
        let pattern = format!("%\"path\":\"{}\"%", escape_like(file_path));
        self.query_receipts("SELECT receipt_json FROM change_receipts WHERE receipt_json LIKE ?1 ESCAPE '\\' ORDER BY timestamp DESC", params![pattern])
    }

    /// Persists a workflow event, replacing any existing event with the same ID.
    pub fn save_event(&self, event: &WorkflowEvent) -> Result<()> {
        let conn = self.lock()?;
        let json = serde_json::to_string(event)?;
        conn.execute(
            "INSERT OR REPLACE INTO workflow_events (id, timestamp, event_type, details_json) VALUES (?1, ?2, ?3, ?4)",
            params![event.id, event.timestamp.to_rfc3339(), format!("{:?}", event.event_type), json],
        )?;
        Ok(())
    }

    pub fn events_since(&self, since: chrono::DateTime<chrono::Utc>) -> Result<Vec<WorkflowEvent>> {
        self.query_events(
            "SELECT details_json FROM workflow_events WHERE timestamp >= ?1 ORDER BY timestamp ASC",
            params![since.to_rfc3339()],
        )
    }
}
