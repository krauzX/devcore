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
    pub fn list_for_semester(conn: &Connection, semester_id: &str) -> Vec<Course> {
        let mut stmt = conn
            .prepare("SELECT id, semester_id, name, code, credits FROM courses WHERE semester_id = ?1 ORDER BY code")
            .unwrap();
        let rows = stmt
            .query_map(params![semester_id], |row| {
                Ok(Course {
                    id: row.get(0)?,
                    semester_id: row.get(1)?,
                    name: row.get(2)?,
                    code: row.get(3)?,
                    credits: row.get(4)?,
                })
            })
            .unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn add(conn: &Connection, course: &Course) {
        conn.execute(
            "INSERT INTO courses (id, semester_id, name, code, credits) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                course.id,
                course.semester_id,
                course.name,
                course.code,
                course.credits,
            ],
        )
        .unwrap();
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
}
