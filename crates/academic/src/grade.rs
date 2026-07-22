use rusqlite::{params, Connection};

#[derive(Debug, Clone)]
pub struct GradeEntry {
    pub id: String,
    pub course_id: String,
    pub semester_id: String,
    pub grade: String,
}

pub fn grade_to_points(grade: &str) -> f64 {
    match grade {
        "O" => 10.0,
        "A+" => 9.0,
        "A" => 8.0,
        "B+" => 7.0,
        "B" => 6.0,
        "C" => 5.0,
        "F" => 0.0,
        _ => 0.0,
    }
}

impl GradeEntry {
    pub fn compute_sgpa(conn: &Connection, semester_id: &str) -> Option<f64> {
        let mut stmt = conn
            .prepare(
                "SELECT g.grade, c.credits \
                 FROM grades g \
                 JOIN courses c ON g.course_id = c.id \
                 WHERE g.semester_id = ?1",
            )
            .ok()?;
        let rows = stmt
            .query_map(params![semester_id], |row| {
                let grade: String = row.get(0)?;
                let credits: i32 = row.get(1)?;
                Ok((grade, credits))
            })
            .ok()?;

        let mut total_points = 0.0;
        let mut total_credits = 0;

        for row in rows {
            if let Ok((grade, credits)) = row {
                total_points += grade_to_points(&grade) * credits as f64;
                total_credits += credits;
            }
        }

        if total_credits == 0 {
            None
        } else {
            Some(total_points / total_credits as f64)
        }
    }

    pub fn add(conn: &Connection, entry: &GradeEntry) {
        conn.execute(
            "INSERT INTO grades (id, course_id, semester_id, grade) VALUES (?1, ?2, ?3, ?4)",
            params![entry.id, entry.course_id, entry.semester_id, entry.grade],
        )
        .unwrap();
    }
}
