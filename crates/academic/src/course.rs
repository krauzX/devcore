use rusqlite::{params, Connection};

#[derive(Debug, Clone)]
pub struct Course {
    pub id: String,
    pub semester_id: String,
    pub name: String,
    pub code: String,
    pub credits: i32,
}

impl Course {
    pub fn list_for_semester(conn: &Connection, semester_id: &str) -> Result<Vec<Course>, rusqlite::Error> {
        let mut stmt = conn
            .prepare("SELECT id, semester_id, name, code, credits FROM courses WHERE semester_id = ?1 ORDER BY code")?;
        let rows = stmt
            .query_map(params![semester_id], |row| {
                Ok(Course {
                    id: row.get(0)?,
                    semester_id: row.get(1)?,
                    name: row.get(2)?,
                    code: row.get(3)?,
                    credits: row.get(4)?,
                })
            })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn add(conn: &Connection, course: &Course) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO courses (id, semester_id, name, code, credits) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                course.id,
                course.semester_id,
                course.name,
                course.code,
                course.credits,
            ],
        )?;
        Ok(())
    }

    pub fn get(conn: &Connection, id: &str) -> Option<Course> {
        let mut stmt = conn
            .prepare("SELECT id, semester_id, name, code, credits FROM courses WHERE id = ?1")
            .ok()?;
        let mut rows = stmt
            .query_map(params![id], |row| {
                Ok(Course {
                    id: row.get(0)?,
                    semester_id: row.get(1)?,
                    name: row.get(2)?,
                    code: row.get(3)?,
                    credits: row.get(4)?,
                })
            })
            .ok()?;
        rows.next().and_then(|r| r.ok())
    }

    pub fn find_by_code(conn: &Connection, code: &str) -> Option<Course> {
        let mut stmt = conn
            .prepare("SELECT id, semester_id, name, code, credits FROM courses WHERE code = ?1")
            .ok()?;
        let mut rows = stmt
            .query_map(params![code], |row| {
                Ok(Course {
                    id: row.get(0)?,
                    semester_id: row.get(1)?,
                    name: row.get(2)?,
                    code: row.get(3)?,
                    credits: row.get(4)?,
                })
            })
            .ok()?;
        rows.next().and_then(|r| r.ok())
    }

    pub fn count_for_semester(conn: &Connection, semester_id: &str) -> i32 {
        conn.query_row(
            "SELECT COUNT(*) FROM courses WHERE semester_id = ?1",
            params![semester_id],
            |row| row.get::<_, i32>(0),
        )
        .unwrap_or(0)
    }

    pub fn total_credits_for_semester(conn: &Connection, semester_id: &str) -> i32 {
        conn.query_row(
            "SELECT COALESCE(SUM(credits), 0) FROM courses WHERE semester_id = ?1",
            params![semester_id],
            |row| row.get::<_, i32>(0),
        )
        .unwrap_or(0)
    }
}
