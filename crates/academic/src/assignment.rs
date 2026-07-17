use anyhow::Result;
use chrono::NaiveDate;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub id: String,
    pub course_id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: NaiveDate,
    pub status: AssignmentStatus,
    pub marks_obtained: Option<u32>,
    pub marks_total: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AssignmentStatus {
    Pending,
    InProgress,
    Submitted,
    Graded,
    Late,
}

impl std::fmt::Display for AssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::InProgress => write!(f, "In Progress"),
            Self::Submitted => write!(f, "Submitted"),
            Self::Graded => write!(f, "Graded"),
            Self::Late => write!(f, "Late"),
        }
    }
}

impl Assignment {
    pub fn list_for_course(conn: &Connection, course_id: &str) -> Result<Vec<Assignment>> {
        let mut stmt = conn.prepare(
            "SELECT id, course_id, title, description, due_date, status, marks_obtained, marks_total
             FROM assignments WHERE course_id = ?1 ORDER BY due_date",
        )?;

        let rows = stmt.query_map(params![course_id], |row| {
            let status_str: String = row.get(5)?;
            let status = match status_str.as_str() {
                "in_progress" => AssignmentStatus::InProgress,
                "submitted" => AssignmentStatus::Submitted,
                "graded" => AssignmentStatus::Graded,
                "late" => AssignmentStatus::Late,
                _ => AssignmentStatus::Pending,
            };
            Ok(Assignment {
                id: row.get(0)?,
                course_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                due_date: NaiveDate::parse_from_str(&row.get::<_, String>(4)?, "%Y-%m-%d")
                    .unwrap_or_default(),
                status,
                marks_obtained: row.get(6)?,
                marks_total: row.get(7)?,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn upcoming(conn: &Connection, days: u32) -> Result<Vec<Assignment>> {
        let today = chrono::Utc::now().date_naive();
        let deadline = today + chrono::Duration::days(days as i64);

        let mut stmt = conn.prepare(
            "SELECT id, course_id, title, description, due_date, status, marks_obtained, marks_total
             FROM assignments WHERE due_date >= ?1 AND due_date <= ?2 AND status != 'graded'
             ORDER BY due_date",
        )?;

        let rows = stmt.query_map(params![today.to_string(), deadline.to_string()], |row| {
            let status_str: String = row.get(5)?;
            let status = match status_str.as_str() {
                "in_progress" => AssignmentStatus::InProgress,
                "submitted" => AssignmentStatus::Submitted,
                "graded" => AssignmentStatus::Graded,
                "late" => AssignmentStatus::Late,
                _ => AssignmentStatus::Pending,
            };
            Ok(Assignment {
                id: row.get(0)?,
                course_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                due_date: NaiveDate::parse_from_str(&row.get::<_, String>(4)?, "%Y-%m-%d")
                    .unwrap_or_default(),
                status,
                marks_obtained: row.get(6)?,
                marks_total: row.get(7)?,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}
