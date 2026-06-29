use crate::models::*;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS change_receipts (
    id TEXT PRIMARY KEY,
    commit_oid TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    is_ai_generated INTEGER NOT NULL DEFAULT 0,
    ai_source TEXT,
    intent TEXT NOT NULL DEFAULT '',
    risk_score INTEGER NOT NULL DEFAULT 0,
    receipt_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS workflow_events (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    event_type TEXT NOT NULL,
    details_json TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_receipts_commit ON change_receipts(commit_oid);
CREATE INDEX IF NOT EXISTS idx_receipts_timestamp ON change_receipts(timestamp);
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON workflow_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_events_type ON workflow_events(event_type);
"#;

fn escape_like_pattern(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// Persistent storage backed by SQLite for change receipts and workflow events.
pub struct Store {
    conn: Mutex<Connection>,
    _path: PathBuf,
}

impl Store {
    /// Opens or creates the database at `.devcore/devcore.db` inside the project root.
    /// Creates the `.devcore` directory and schema if they do not exist.
    pub fn open(project_root: &Path) -> Result<Self> {
        let db_dir = project_root.join(".devcore");
        std::fs::create_dir_all(&db_dir)?;
        let db_path = db_dir.join("devcore.db");

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database at {}", db_path.display()))?;

        conn.execute_batch(SCHEMA)?;

        Ok(Self {
            conn: Mutex::new(conn),
            _path: db_path,
        })
    }

    /// Persists a change receipt, replacing any existing receipt with the same ID.
    pub fn save_receipt(&self, receipt: &ChangeReceipt) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let json = serde_json::to_string(receipt)?;

        conn.execute(
            "INSERT OR REPLACE INTO change_receipts
             (id, commit_oid, timestamp, is_ai_generated, ai_source, intent, risk_score, receipt_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                receipt.id,
                receipt.commit_oid,
                receipt.timestamp.to_rfc3339(),
                receipt.is_ai_generated as i32,
                receipt.ai_source.as_ref().map(|s| format!("{:?}", s)),
                receipt.intent,
                receipt.risk_score,
                json,
            ],
        )?;

        Ok(())
    }

    /// Retrieves a change receipt by its associated commit OID.
    /// Returns `Ok(None)` if no receipt exists for the given OID.
    pub fn get_receipt(&self, commit_oid: &str) -> Result<Option<ChangeReceipt>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT receipt_json FROM change_receipts WHERE commit_oid = ?1",
        )?;

        let mut rows = stmt.query_map(params![commit_oid], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;

        match rows.next() {
            Some(Ok(json)) => {
                let receipt: ChangeReceipt = serde_json::from_str(&json)?;
                Ok(Some(receipt))
            }
            _ => Ok(None),
        }
    }

    /// Returns the most recent change receipts, ordered by timestamp descending.
    pub fn recent_receipts(&self, limit: usize) -> Result<Vec<ChangeReceipt>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT receipt_json FROM change_receipts ORDER BY timestamp DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;

        let mut receipts = Vec::new();
        for row in rows {
            let json = row?;
            let receipt: ChangeReceipt = serde_json::from_str(&json)?;
            receipts.push(receipt);
        }

        Ok(receipts)
    }

    /// Returns all change receipts that touch the given file path, ordered by timestamp descending.
    pub fn receipts_for_file(&self, file_path: &str) -> Result<Vec<ChangeReceipt>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT receipt_json FROM change_receipts
             WHERE receipt_json LIKE ?1 ESCAPE '\\'
             ORDER BY timestamp DESC",
        )?;

        let escaped = escape_like_pattern(file_path);
        let pattern = format!("%\"path\":\"{}\"%", escaped);
        let rows = stmt.query_map(params![pattern], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;

        let mut receipts = Vec::new();
        for row in rows {
            let json = row?;
            let receipt: ChangeReceipt = serde_json::from_str(&json)?;
            receipts.push(receipt);
        }

        Ok(receipts)
    }

    /// Persists a workflow event, replacing any existing event with the same ID.
    pub fn save_event(&self, event: &WorkflowEvent) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let json = serde_json::to_string(event)?;

        conn.execute(
            "INSERT OR REPLACE INTO workflow_events
             (id, timestamp, event_type, details_json)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                event.id,
                event.timestamp.to_rfc3339(),
                format!("{:?}", event.event_type),
                json,
            ],
        )?;

        Ok(())
    }

    /// Returns all workflow events since the given timestamp, ordered by timestamp ascending.
    pub fn events_since(&self, since: chrono::DateTime<chrono::Utc>) -> Result<Vec<WorkflowEvent>> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT details_json FROM workflow_events
             WHERE timestamp >= ?1
             ORDER BY timestamp ASC",
        )?;

        let rows = stmt.query_map(params![since.to_rfc3339()], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;

        let mut events = Vec::new();
        for row in rows {
            let json = row?;
            let event: WorkflowEvent = serde_json::from_str(&json)?;
            events.push(event);
        }

        Ok(events)
    }
}
