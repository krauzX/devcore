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

pub fn score_to_grade(obtained: f64, total: f64) -> &'static str {
    if total <= 0.0 {
        return "F";
    }
    let pct = (obtained / total) * 100.0;
    if pct >= 90.0 {
        "O"
    } else if pct >= 80.0 {
        "A+"
    } else if pct >= 70.0 {
        "A"
    } else if pct >= 60.0 {
        "B+"
    } else if pct >= 50.0 {
        "B"
    } else if pct >= 40.0 {
        "C"
    } else {
        "F"
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

        for (grade, credits) in rows.flatten() {
            total_points += grade_to_points(&grade) * credits as f64;
            total_credits += credits;
        }

        if total_credits == 0 {
            None
        } else {
            Some(total_points / total_credits as f64)
        }
    }

    pub fn add(conn: &Connection, entry: &GradeEntry) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO grades (id, course_id, semester_id, grade) VALUES (?1, ?2, ?3, ?4)",
            params![entry.id, entry.course_id, entry.semester_id, entry.grade],
        )?;
        Ok(())
    }

    pub fn compute_cgpa(conn: &Connection) -> Option<f64> {
        let mut stmt = conn
            .prepare(
                "SELECT g.grade, c.credits \
                 FROM grades g \
                 JOIN courses c ON g.course_id = c.id",
            )
            .ok()?;
        let rows = stmt
            .query_map([], |row| {
                let grade: String = row.get(0)?;
                let credits: i32 = row.get(1)?;
                Ok((grade, credits))
            })
            .ok()?;

        let mut total_points = 0.0;
        let mut total_credits = 0;

        for (grade, credits) in rows.flatten() {
            total_points += grade_to_points(&grade) * credits as f64;
            total_credits += credits;
        }

        if total_credits == 0 {
            None
        } else {
            Some(total_points / total_credits as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grade_to_points_o() {
        assert_eq!(grade_to_points("O"), 10.0);
    }

    #[test]
    fn test_grade_to_points_aplus() {
        assert_eq!(grade_to_points("A+"), 9.0);
    }

    #[test]
    fn test_grade_to_points_f() {
        assert_eq!(grade_to_points("F"), 0.0);
    }

    #[test]
    fn test_grade_to_points_percentage() {
        assert_eq!(score_to_grade(95.0, 100.0), "O");
        assert_eq!(score_to_grade(85.0, 100.0), "A+");
        assert_eq!(score_to_grade(75.0, 100.0), "A");
        assert_eq!(score_to_grade(65.0, 100.0), "B+");
        assert_eq!(score_to_grade(55.0, 100.0), "B");
        assert_eq!(score_to_grade(45.0, 100.0), "C");
        assert_eq!(score_to_grade(30.0, 100.0), "F");
    }
}
