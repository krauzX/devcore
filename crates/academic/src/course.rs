use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// A course within a semester.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    /// Unique course identifier (e.g. "sem-5-CS301")
    pub id: String,
    /// ID of the semester this course belongs to
    pub semester_id: String,
    /// Course code (e.g. "CS301")
    pub code: String,
    /// Full course name
    pub name: String,
    /// Credit hours
    pub credits: u8,
    /// Course type ("theory" or "lab")
    pub course_type: String,
    /// Instructor name, if known
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

    /// Returns the total credits for all courses in a semester.
    pub fn total_credits(conn: &Connection, semester_id: &str) -> Result<u8> {
        let mut stmt =
            conn.prepare("SELECT COALESCE(SUM(credits), 0) FROM courses WHERE semester_id = ?1")?;
        let credits: u8 = stmt.query_row(params![semester_id], |row| row.get(0))?;
        Ok(credits)
    }
}
