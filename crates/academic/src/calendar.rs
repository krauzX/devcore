use anyhow::Result;
use chrono::NaiveDate;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// An academic event such as an exam, assignment deadline, or holiday.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicEvent {
    /// Unique event identifier
    pub id: String,
    /// Event title
    pub title: String,
    /// Type of event
    pub event_type: EventType,
    /// Date of the event
    pub date: NaiveDate,
    /// Optional time of day (e.g. "10:00")
    pub time: Option<String>,
    /// Associated course ID, if applicable
    pub course_id: Option<String>,
    /// Optional notes
    pub notes: Option<String>,
}

/// Categories of academic events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Examination
    Exam,
    /// Assignment deadline
    Assignment,
    /// Laboratory session
    Lab,
    /// Lecture
    Lecture,
    /// Holiday
    Holiday,
    /// Submission deadline
    Submission,
    /// Presentation
    Presentation,
    /// Uncategorized event
    Other,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exam => write!(f, "Exam"),
            Self::Assignment => write!(f, "Assignment"),
            Self::Lab => write!(f, "Lab"),
            Self::Lecture => write!(f, "Lecture"),
            Self::Holiday => write!(f, "Holiday"),
            Self::Submission => write!(f, "Submission"),
            Self::Presentation => write!(f, "Presentation"),
            Self::Other => write!(f, "Other"),
        }
    }
}

impl AcademicEvent {
    /// Returns upcoming events within the given number of days from today.
    pub fn upcoming(conn: &Connection, days: u32) -> Result<Vec<AcademicEvent>> {
        let today = chrono::Utc::now().date_naive();
        let deadline = today + chrono::Duration::days(days as i64);

        let mut stmt = conn.prepare(
            "SELECT id, title, event_type, date, time, course_id, notes
             FROM events WHERE date >= ?1 AND date <= ?2 ORDER BY date, time",
        )?;

        let rows = stmt.query_map(params![today.to_string(), deadline.to_string()], |row| {
            let type_str: String = row.get(2)?;
            let event_type = match type_str.as_str() {
                "exam" => EventType::Exam,
                "assignment" => EventType::Assignment,
                "lab" => EventType::Lab,
                "lecture" => EventType::Lecture,
                "holiday" => EventType::Holiday,
                "submission" => EventType::Submission,
                "presentation" => EventType::Presentation,
                _ => EventType::Other,
            };

            Ok(AcademicEvent {
                id: row.get(0)?,
                title: row.get(1)?,
                event_type,
                date: NaiveDate::parse_from_str(&row.get::<_, String>(3)?, "%Y-%m-%d")
                    .unwrap_or_default(),
                time: row.get(4)?,
                course_id: row.get(5)?,
                notes: row.get(6)?,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Returns events occurring within the next 7 days.
    pub fn this_week(conn: &Connection) -> Result<Vec<AcademicEvent>> {
        Self::upcoming(conn, 7)
    }
}
