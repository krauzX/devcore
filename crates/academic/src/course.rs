use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: String,
    pub semester_id: String,
    pub code: String,
    pub name: String,
    pub credits: u8,
    pub course_type: String,
    pub instructor: Option<String>,
}

impl Course {
    /// Lists all courses for a given semester, ordered by course code.
    pub fn list_for_semester(conn: &Connection, semester_id: &str) -> Result<Vec<Course>> {
        let mut stmt = conn.prepare(
            "SELECT id, semester_id, code, name, credits, type, instructor
             FROM courses WHERE semester_id = ?1 ORDER BY code",
        )?;

        let rows = stmt.query_map(params![semester_id], |row| {
            Ok(Course {
                id: row.get(0)?,
                semester_id: row.get(1)?,
                code: row.get(2)?,
                name: row.get(3)?,
                credits: row.get(4)?,
                course_type: row.get(5)?,
                instructor: row.get(6)?,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn total_credits(conn: &Connection, semester_id: &str) -> Result<u8> {
        let mut stmt =
            conn.prepare("SELECT COALESCE(SUM(credits), 0) FROM courses WHERE semester_id = ?1")?;
        let credits: u8 = stmt.query_row(params![semester_id], |row| row.get(0))?;
        Ok(credits)
    }
}
