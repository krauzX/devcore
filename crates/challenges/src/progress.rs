use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ChallengeProgress {
    pub problem_id: String,
    pub pack_id: String,
    pub solved: bool,
    pub attempts: i32,
    pub hints_used: i32,
    pub time_spent_secs: i64,
    pub last_attempt_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Error)]
pub enum RecordAttemptError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
}

const CHALLENGE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS challenge_progress (
    problem_id TEXT NOT NULL,
    pack_id TEXT NOT NULL,
    solved INTEGER NOT NULL DEFAULT 0,
    attempts INTEGER NOT NULL DEFAULT 0,
    hints_used INTEGER NOT NULL DEFAULT 0,
    time_spent_secs INTEGER NOT NULL DEFAULT 0,
    last_attempt_at TEXT,
    PRIMARY KEY (problem_id, pack_id)
);
"#;

pub fn init_challenge_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(CHALLENGE_SCHEMA)?;
    Ok(())
}

pub fn get_progress(
    conn: &Connection,
    problem_id: &str,
    pack_id: &str,
) -> Result<Option<ChallengeProgress>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT problem_id, pack_id, solved, attempts, hints_used, time_spent_secs, last_attempt_at
         FROM challenge_progress
         WHERE problem_id = ?1 AND pack_id = ?2",
    )?;
    let mut rows = stmt.query_map(params![problem_id, pack_id], |row| {
        Ok(ChallengeProgress {
            problem_id: row.get(0)?,
            pack_id: row.get(1)?,
            solved: row.get::<_, i32>(2)? != 0,
            attempts: row.get(3)?,
            hints_used: row.get(4)?,
            time_spent_secs: row.get(5)?,
            last_attempt_at: row
                .get::<_, Option<String>>(6)?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
        })
    })?;
    match rows.next() {
        Some(Ok(progress)) => Ok(Some(progress)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

pub fn record_attempt(
    conn: &Connection,
    problem_id: &str,
    pack_id: &str,
    solved: bool,
    time_secs: i64,
) -> Result<ChallengeProgress, RecordAttemptError> {
    let now = Utc::now().to_rfc3339();
    let solved_val: i32 = if solved { 1 } else { 0 };
    conn.execute(
        "INSERT INTO challenge_progress (problem_id, pack_id, solved, attempts, hints_used, time_spent_secs, last_attempt_at)
         VALUES (?1, ?2, ?3, 1, 0, ?4, ?5)
         ON CONFLICT(problem_id, pack_id) DO UPDATE SET
             solved = MAX(challenge_progress.solved, excluded.solved),
             attempts = challenge_progress.attempts + 1,
             time_spent_secs = challenge_progress.time_spent_secs + excluded.time_spent_secs,
             last_attempt_at = excluded.last_attempt_at",
        params![problem_id, pack_id, solved_val, time_secs, now],
    )?;
    get_progress(conn, problem_id, pack_id)?
        .ok_or_else(|| RecordAttemptError::Database(rusqlite::Error::QueryReturnedNoRows))
}

pub fn record_hint(
    conn: &Connection,
    problem_id: &str,
    pack_id: &str,
) -> Result<ChallengeProgress, RecordAttemptError> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO challenge_progress (problem_id, pack_id, solved, attempts, hints_used, time_spent_secs, last_attempt_at)
         VALUES (?1, ?2, 0, 0, 1, 0, ?3)
         ON CONFLICT(problem_id, pack_id) DO UPDATE SET
             hints_used = challenge_progress.hints_used + 1,
             last_attempt_at = excluded.last_attempt_at",
        params![problem_id, pack_id, now],
    )?;
    get_progress(conn, problem_id, pack_id)?
        .ok_or_else(|| RecordAttemptError::Database(rusqlite::Error::QueryReturnedNoRows))
}

pub fn get_pack_stats(
    conn: &Connection,
    pack_id: &str,
) -> Result<(i64, i64, i64), rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT
            COUNT(CASE WHEN solved = 1 THEN 1 END) as solved,
            COUNT(*) as total,
            COALESCE(SUM(time_spent_secs), 0) as total_time
         FROM challenge_progress
         WHERE pack_id = ?1",
    )?;
    let mut rows = stmt.query_map(params![pack_id], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))
    })?;
    match rows.next() {
        Some(Ok(stats)) => Ok(stats),
        Some(Err(e)) => Err(e),
        None => Ok((0, 0, 0)),
    }
}
