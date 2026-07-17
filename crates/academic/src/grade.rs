use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeEntry {
    pub id: String,
    pub course_id: String,
    pub exam_type: String,
    pub marks_obtained: f64,
    pub marks_total: f64,
    pub grade: Option<String>,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CgpaReport {
    pub semester: u8,
    pub sgpa: f64,
    pub cgpa: f64,
    pub total_credits: u8,
    pub courses: Vec<CourseGrade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseGrade {
    pub code: String,
    pub name: String,
    pub credits: u8,
    pub grade_points: f64,
    pub grade: String,
}

impl GradeEntry {
    pub fn compute_sgpa(conn: &Connection, semester_id: &str) -> Result<Option<f64>> {
        let grades = Self::list_for_semester(conn, semester_id)?;

        if grades.is_empty() {
            return Ok(None);
        }

        let mut total_points = 0.0;
        let mut total_credits = 0u32;

        for grade in &grades {
            // Get course credits
            let credits: u32 = conn.query_row(
                "SELECT credits FROM courses WHERE id = ?1",
                params![grade.course_id],
                |r| r.get(0),
            )?;

            let gp = grade_to_points(&grade.grade.clone().unwrap_or_default());
            total_points += gp * credits as f64;
            total_credits += credits;
        }

        if total_credits == 0 {
            return Ok(None);
        }

        Ok(Some(total_points / total_credits as f64))
    }

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

pub fn points_to_grade(points: f64) -> &'static str {
    match points as u32 {
        10 => "O",
        9 => "A+",
        8 => "A",
        7 => "B+",
        6 => "B",
        5 => "C",
        _ => "F",
    }
}
