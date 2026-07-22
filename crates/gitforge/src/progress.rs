use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProgressError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
}

pub const XP_PER_LEVEL: u32 = 100;
pub const MAX_LEVEL: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillAxis {
    CommitHygiene,
    Testing,
    Documentation,
    CodeReview,
    Architecture,
}

impl SkillAxis {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CommitHygiene => "commit_hygiene",
            Self::Testing => "testing",
            Self::Documentation => "documentation",
            Self::CodeReview => "code_review",
            Self::Architecture => "architecture",
        }
    }

    pub fn all() -> &'static [SkillAxis] {
        &[
            Self::CommitHygiene,
            Self::Testing,
            Self::Documentation,
            Self::CodeReview,
            Self::Architecture,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProgress {
    pub axis: SkillAxis,
    pub xp: u32,
    pub level: u32,
}

impl SkillProgress {
    fn from_xp(axis: SkillAxis, xp: u32) -> Self {
        let level = std::cmp::min(xp / XP_PER_LEVEL, MAX_LEVEL);
        Self { axis, xp, level }
    }
}

pub fn init_skill_schema(conn: &Connection) -> Result<(), ProgressError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS skill_xp (
            axis TEXT PRIMARY KEY NOT NULL,
            xp INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS skill_xp_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            axis TEXT NOT NULL,
            xp INTEGER NOT NULL,
            reason TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    for axis in SkillAxis::all() {
        conn.execute(
            "INSERT OR IGNORE INTO skill_xp (axis, xp) VALUES (?1, 0)",
            params![axis.as_str()],
        )?;
    }

    Ok(())
}

pub fn get_progress(conn: &Connection) -> Result<Vec<SkillProgress>, ProgressError> {
    let mut stmt = conn.prepare("SELECT axis, xp FROM skill_xp")?;
    let rows = stmt.query_map([], |row| {
        let axis_str: String = row.get(0)?;
        let xp: u32 = row.get(1)?;
        Ok((axis_str, xp))
    })?;

    let mut results = Vec::new();
    for row in rows {
        let (axis_str, xp) = row?;
        let axis = match axis_str.as_str() {
            "commit_hygiene" => SkillAxis::CommitHygiene,
            "testing" => SkillAxis::Testing,
            "documentation" => SkillAxis::Documentation,
            "code_review" => SkillAxis::CodeReview,
            "architecture" => SkillAxis::Architecture,
            _ => continue,
        };
        results.push(SkillProgress::from_xp(axis, xp));
    }

    Ok(results)
}

pub fn add_xp(
    conn: &Connection,
    axis: SkillAxis,
    xp: u32,
    reason: &str,
) -> Result<SkillProgress, ProgressError> {
    conn.execute(
        "UPDATE skill_xp SET xp = xp + ?1 WHERE axis = ?2",
        params![xp, axis.as_str()],
    )?;

    conn.execute(
        "INSERT INTO skill_xp_log (axis, xp, reason) VALUES (?1, ?2, ?3)",
        params![axis.as_str(), xp, reason],
    )?;

    let current_xp: u32 = conn.query_row(
        "SELECT xp FROM skill_xp WHERE axis = ?1",
        params![axis.as_str()],
        |row| row.get(0),
    )?;

    Ok(SkillProgress::from_xp(axis, current_xp))
}
