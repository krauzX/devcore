use chrono::NaiveDate;
use devcore_core::DevCoreError;
use rusqlite::{params, Connection};
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
    pub fn open(project_root: &Path) -> Result<Self, DevCoreError> {
        let db_dir = project_root.join(".devcore");
        std::fs::create_dir_all(&db_dir)?;
        let db_path = db_dir.join("academic.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.execute_batch(ACADEMIC_SCHEMA)?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        let needs_seed = store.conn().map(|conn| {
            let count: i32 = conn
                .query_row("SELECT COUNT(*) FROM semesters", [], |row| row.get(0))
                .unwrap_or(0);
            count == 0
        }).unwrap_or(true);
        if needs_seed {
            store.seed_2026_data()?;
        }
        Ok(store)
    }

    pub fn seed_2026_data(&self) -> Result<(), DevCoreError> {
        let conn = self.conn().map_err(DevCoreError::Config)?;
        let tx = conn.unchecked_transaction().map_err(|e| DevCoreError::Config(e.to_string()))?;

        let semesters = vec![
            Semester {
                id: "sem1-2026".into(),
                name: "Semester 1 — 2026 (Jul–Nov)".into(),
                start_date: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2026, 11, 30).unwrap(),
                is_current: true,
            },
            Semester {
                id: "sem2-2027".into(),
                name: "Semester 2 — 2027 (Jan–May)".into(),
                start_date: NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2027, 5, 31).unwrap(),
                is_current: false,
            },
            Semester {
                id: "sem3-2027".into(),
                name: "Semester 3 — 2027 (Jul–Nov)".into(),
                start_date: NaiveDate::from_ymd_opt(2027, 7, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2027, 11, 30).unwrap(),
                is_current: false,
            },
            Semester {
                id: "sem4-2028".into(),
                name: "Semester 4 — 2028 (Jan–May)".into(),
                start_date: NaiveDate::from_ymd_opt(2028, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2028, 5, 31).unwrap(),
                is_current: false,
            },
            Semester {
                id: "sem5-2028".into(),
                name: "Semester 5 — 2028 (Jul–Nov)".into(),
                start_date: NaiveDate::from_ymd_opt(2028, 7, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2028, 11, 30).unwrap(),
                is_current: false,
            },
            Semester {
                id: "sem6-2029".into(),
                name: "Semester 6 — 2029 (Jan–May)".into(),
                start_date: NaiveDate::from_ymd_opt(2029, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2029, 5, 31).unwrap(),
                is_current: false,
            },
            Semester {
                id: "sem7-2029".into(),
                name: "Semester 7 — 2029 (Jul–Nov)".into(),
                start_date: NaiveDate::from_ymd_opt(2029, 7, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2029, 11, 30).unwrap(),
                is_current: false,
            },
            Semester {
                id: "sem8-2030".into(),
                name: "Semester 8 — 2030 (Jan–May)".into(),
                start_date: NaiveDate::from_ymd_opt(2030, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2030, 5, 31).unwrap(),
                is_current: false,
            },
        ];
        for sem in &semesters {
            tx.execute(
                "INSERT OR IGNORE INTO semesters (id, name, start_date, end_date, is_current) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    sem.id,
                    sem.name,
                    sem.start_date.to_string(),
                    sem.end_date.to_string(),
                    sem.is_current as i32,
                ],
            ).map_err(|e| DevCoreError::Config(e.to_string()))?;
        }

        let courses = vec![
            ("cs101-sem1", "sem1-2026", "Programming Fundamentals", "CS101", 4),
            ("ma101-sem1", "sem1-2026", "Engineering Mathematics I", "MA101", 4),
            ("ph101-sem1", "sem1-2026", "Engineering Physics", "PH101", 3),
            ("ee101-sem1", "sem1-2026", "Basic Electrical Engineering", "EE101", 3),
            ("cs102-sem1", "sem1-2026", "Data Structures", "CS102", 4),
            ("cs103-sem1", "sem1-2026", "Digital Logic", "CS103", 3),
            ("hs101-sem1", "sem1-2026", "English Communication", "HS101", 2),
            ("cs104-sem1", "sem1-2026", "Programming Lab", "CS104", 1),
        ];
        for (id, sem_id, name, code, credits) in &courses {
            tx.execute(
                "INSERT OR IGNORE INTO courses (id, semester_id, name, code, credits) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, sem_id, name, code, credits],
            ).map_err(|e| DevCoreError::Config(e.to_string()))?;
        }

        tx.commit().map_err(|e| DevCoreError::Config(e.to_string()))?;
        Ok(())
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
                    start_date: row.get::<_, String>(2)?.parse().ok()
                        .filter(|d| *d > NaiveDate::from_ymd_opt(2000, 1, 1).unwrap())
                        .unwrap_or_default(),
                    end_date: row.get::<_, String>(3)?.parse().ok()
                        .filter(|d| *d > NaiveDate::from_ymd_opt(2000, 1, 1).unwrap())
                        .unwrap_or_default(),
                    is_current: row.get::<_, i32>(4)? != 0,
                })
            })
            .ok()?;
        rows.next().and_then(|r| r.ok())
    }

    pub fn list_semesters(&self) -> Result<Vec<Semester>, DevCoreError> {
        let conn = self.conn().map_err(DevCoreError::Config)?;
        let mut stmt = conn
            .prepare("SELECT id, name, start_date, end_date, is_current FROM semesters ORDER BY start_date DESC")?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Semester {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    start_date: row.get::<_, String>(2)?.parse().ok()
                        .filter(|d| *d > NaiveDate::from_ymd_opt(2000, 1, 1).unwrap())
                        .unwrap_or_default(),
                    end_date: row.get::<_, String>(3)?.parse().ok()
                        .filter(|d| *d > NaiveDate::from_ymd_opt(2000, 1, 1).unwrap())
                        .unwrap_or_default(),
                    is_current: row.get::<_, i32>(4)? != 0,
                })
            })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn set_current_semester(&self, id: &str) -> Result<(), DevCoreError> {
        let conn = self.conn().map_err(DevCoreError::Config)?;
        conn.execute("UPDATE semesters SET is_current = 0", [])?;
        conn.execute(
            "UPDATE semesters SET is_current = 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn add_semester(&self, sem: &Semester) -> Result<(), DevCoreError> {
        let conn = self.conn().map_err(DevCoreError::Config)?;
        conn.execute(
            "INSERT OR IGNORE INTO semesters (id, name, start_date, end_date, is_current) VALUES (?1, ?2, ?3, ?4, ?5)",
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
