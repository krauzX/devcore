use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// A single grade entry recorded for a course.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeEntry {
    /// Unique grade entry identifier
    pub id: String,
    /// ID of the course this grade belongs to
    pub course_id: String,
    /// Type of examination (e.g. "midterm", "final", "quiz")
    pub exam_type: String,
    /// Marks obtained by the student
    pub marks_obtained: f64,
    /// Maximum marks possible
    pub marks_total: f64,
    /// Letter grade, if assigned
    pub grade: Option<String>,
    /// ISO 8601 timestamp when the grade was recorded
    pub recorded_at: String,
}

/// Cumulative grade report for a student across semesters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CgpaReport {
    /// Semester number
    pub semester: u8,
    /// Semester GPA
    pub sgpa: f64,
    /// Cumulative GPA up to this semester
    pub cgpa: f64,
    /// Total credits earned in this semester
    pub total_credits: u8,
    /// Per-course grade breakdown
    pub courses: Vec<CourseGrade>,
}

/// Grade information for a single course.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseGrade {
    /// Course code (e.g. "CS301")
    pub code: String,
    /// Course name
    pub name: String,
    /// Credit hours
    pub credits: u8,
    /// Numeric grade points on the 10-point scale
    pub grade_points: f64,
    /// Letter grade
    pub grade: String,
}

impl GradeEntry {
    /// Computes the SGPA for a given semester using grade points and credits.
    pub fn compute_sgpa(conn: &Connection, semester_id: &str) -> Result<Option<f64>> {
        let mut stmt = conn.prepare(
            "SELECT g.grade, c.credits
             FROM grades g
             JOIN courses c ON g.course_id = c.id
             WHERE c.semester_id = ?1",
        )?;

        let mut total_points = 0.0;
        let mut total_credits = 0u32;

        let rows = stmt.query_map(params![semester_id], |row| {
            let grade: Option<String> = row.get(0)?;
            let credits: u32 = row.get(1)?;
            Ok((grade, credits))
        })?;

        for row in rows {
            let (grade, credits) = row?;
            let gp = grade_to_points(&grade.unwrap_or_default());
            total_points += gp * credits as f64;
            total_credits += credits;
        }

        if total_credits == 0 {
            return Ok(None);
        }

        Ok(Some(total_points / total_credits as f64))
    }

    /// Lists all grade entries for courses in a given semester.
    pub fn list_for_semester(conn: &Connection, semester_id: &str) -> Result<Vec<GradeEntry>> {
        let mut stmt = conn.prepare(
            "SELECT g.id, g.course_id, g.exam_type, g.marks_obtained, g.marks_total,
                    g.grade, g.recorded_at
             FROM grades g
             JOIN courses c ON g.course_id = c.id
             WHERE c.semester_id = ?1",
        )?;

        let rows = stmt.query_map(params![semester_id], |row| {
            Ok(GradeEntry {
                id: row.get(0)?,
                course_id: row.get(1)?,
                exam_type: row.get(2)?,
                marks_obtained: row.get(3)?,
                marks_total: row.get(4)?,
                grade: row.get(5)?,
                recorded_at: row.get(6)?,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}

/// Converts a letter grade or percentage string to numeric grade points (0.0–10.0).
pub fn grade_to_points(grade: &str) -> f64 {
    match grade.to_uppercase().as_str() {
        "O" | "O+" => 10.0,
        "A+" => 9.0,
        "A" => 8.0,
        "B+" => 7.0,
        "B" => 6.0,
        "C" => 5.0,
        "F" | "FAIL" => 0.0,
        _ => {
            // Try parsing as percentage
            if let Ok(pct) = grade.parse::<f64>() {
                if pct >= 90.0 {
                    10.0
                } else if pct >= 80.0 {
                    9.0
                } else if pct >= 70.0 {
                    8.0
                } else if pct >= 60.0 {
                    7.0
                } else if pct >= 50.0 {
                    6.0
                } else if pct >= 40.0 {
                    5.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        }
    }
}
