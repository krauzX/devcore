use chrono::NaiveDate;
use rusqlite::{params, Connection, Result as SqlResult};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

#[derive(Debug, Clone)]
pub struct Semester {
    pub id: String,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub is_current: bool,
}

pub struct SemesterStore {
    conn: Mutex<Connection>,
}

const ACADEMIC_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS semesters (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    is_current INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS courses (
    id TEXT PRIMARY KEY,
    semester_id TEXT NOT NULL,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    credits INTEGER NOT NULL,
    FOREIGN KEY (semester_id) REFERENCES semesters(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_courses_semester ON courses(semester_id);

CREATE TABLE IF NOT EXISTS grades (
    id TEXT PRIMARY KEY,
    course_id TEXT NOT NULL,
    semester_id TEXT NOT NULL,
    grade TEXT NOT NULL,
    exam_name TEXT,
    score REAL,
    total REAL,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE,
    FOREIGN KEY (semester_id) REFERENCES semesters(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_grades_semester ON grades(semester_id);
CREATE INDEX IF NOT EXISTS idx_grades_course ON grades(course_id);

CREATE TABLE IF NOT EXISTS deadlines (
    id TEXT PRIMARY KEY,
    semester_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    due_date TEXT NOT NULL,
    completed INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (semester_id) REFERENCES semesters(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_deadlines_semester ON deadlines(semester_id);
CREATE INDEX IF NOT EXISTS idx_deadlines_due ON deadlines(due_date);
"#;

impl SemesterStore {
    pub fn open(project_root: &Path) -> SqlResult<Self> {
        let db_dir = project_root.join(".devcore");
        std::fs::create_dir_all(&db_dir)
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let db_path = db_dir.join("academic.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.execute_batch(ACADEMIC_SCHEMA)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn conn(&self) -> Result<MutexGuard<'_, Connection>, String> {
        self.conn
            .lock()
            .map_err(|e| format!("lock poisoned: {}", e))
    }

    pub fn current_semester(&self) -> Option<Semester> {
        let conn = self.conn().ok()?;
        let mut stmt = conn
            .prepare("SELECT id, name, start_date, end_date, is_current FROM semesters WHERE is_current = 1")
            .ok()?;
        let mut rows = stmt
            .query_map([], |row| {
                Ok(Semester {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    start_date: row.get::<_, String>(2)?.parse().unwrap_or_default(),
                    end_date: row.get::<_, String>(3)?.parse().unwrap_or_default(),
                    is_current: row.get::<_, i32>(4)? != 0,
                })
            })
            .ok()?;
        rows.next().and_then(|r| r.ok())
    }

    pub fn list_semesters(&self) -> Result<Vec<Semester>, rusqlite::Error> {
        let conn = self.conn().map_err(rusqlite::Error::InvalidParameterName)?;
        let mut stmt = conn
            .prepare("SELECT id, name, start_date, end_date, is_current FROM semesters ORDER BY start_date DESC")?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Semester {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    start_date: row.get::<_, String>(2)?.parse().unwrap_or_default(),
                    end_date: row.get::<_, String>(3)?.parse().unwrap_or_default(),
                    is_current: row.get::<_, i32>(4)? != 0,
                })
            })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn set_current_semester(&self, id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn().map_err(rusqlite::Error::InvalidParameterName)?;
        conn.execute("UPDATE semesters SET is_current = 0", [])?;
        conn.execute(
            "UPDATE semesters SET is_current = 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn add_semester(&self, sem: &Semester) -> Result<(), rusqlite::Error> {
        let conn = self.conn().map_err(rusqlite::Error::InvalidParameterName)?;
        conn.execute(
            "INSERT INTO semesters (id, name, start_date, end_date, is_current) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                sem.id,
                sem.name,
                sem.start_date.to_string(),
                sem.end_date.to_string(),
                sem.is_current as i32,
            ],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_semester_store_open() {
        let dir = tempdir().unwrap();
        let store = SemesterStore::open(dir.path()).unwrap();
        let sems = store.list_semesters().unwrap();
        assert!(sems.is_empty());
    }

    #[test]
    fn test_add_and_list_semesters() {
        let dir = tempdir().unwrap();
        let store = SemesterStore::open(dir.path()).unwrap();

        let sem1 = Semester {
            id: "s1".into(),
            name: "Sem 1".into(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 6, 30).unwrap(),
            is_current: false,
        };
        let sem2 = Semester {
            id: "s2".into(),
            name: "Sem 2".into(),
            start_date: NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            is_current: true,
        };

        store.add_semester(&sem1).unwrap();
        store.add_semester(&sem2).unwrap();

        let sems = store.list_semesters().unwrap();
        assert_eq!(sems.len(), 2);

        let current = store.current_semester().unwrap();
        assert_eq!(current.id, "s2");
        assert!(current.is_current);
    }
}
