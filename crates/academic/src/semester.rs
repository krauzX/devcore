use anyhow::Result;
use chrono::{Datelike, NaiveDate, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Semester {
    pub id: String,
    pub number: u8,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicConfig {
    pub institution: String,
    pub program: String,
    pub batch: String,
    pub grading_scale: GradingScale,
    pub total_semesters: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GradingScale {
    Indian10, // 10-point CGPA (IIIT Kottayam standard)
    Indian4,  // 4-point GPA
    Percentage,
    LetterGrade,
}

pub struct SemesterStore {
    conn: Mutex<Connection>,
}

impl SemesterStore {
    /// Get a locked reference to the database connection.
    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }

    pub fn open(project_root: &Path) -> Result<Self> {
        let db_dir = project_root.join(".devcore");
        std::fs::create_dir_all(&db_dir)?;
        let db_path = db_dir.join("academic.db");

        let conn = Connection::open(&db_path)?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS semesters (
                id TEXT PRIMARY KEY,
                number INTEGER NOT NULL,
                name TEXT NOT NULL,
                start_date TEXT NOT NULL,
                end_date TEXT NOT NULL,
                is_current INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS courses (
                id TEXT PRIMARY KEY,
                semester_id TEXT NOT NULL,
                code TEXT NOT NULL,
                name TEXT NOT NULL,
                credits INTEGER NOT NULL,
                type TEXT NOT NULL DEFAULT 'theory',
                instructor TEXT,
                FOREIGN KEY (semester_id) REFERENCES semesters(id)
            );
            CREATE TABLE IF NOT EXISTS assignments (
                id TEXT PRIMARY KEY,
                course_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                due_date TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                marks_obtained INTEGER,
                marks_total INTEGER,
                FOREIGN KEY (course_id) REFERENCES courses(id)
            );
            CREATE TABLE IF NOT EXISTS grades (
                id TEXT PRIMARY KEY,
                course_id TEXT NOT NULL,
                exam_type TEXT NOT NULL,
                marks_obtained REAL NOT NULL,
                marks_total REAL NOT NULL,
                grade TEXT,
                recorded_at TEXT NOT NULL,
                FOREIGN KEY (course_id) REFERENCES courses(id)
            );
            CREATE TABLE IF NOT EXISTS papers (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                authors TEXT,
                venue TEXT,
                year INTEGER,
                doi TEXT,
                arxiv_id TEXT,
                status TEXT NOT NULL DEFAULT 'to_read',
                tags TEXT,
                notes TEXT,
                added_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                event_type TEXT NOT NULL,
                date TEXT NOT NULL,
                time TEXT,
                course_id TEXT,
                notes TEXT,
                FOREIGN KEY (course_id) REFERENCES courses(id)
            );
            ",
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn init_iiit_kottayam(&self) -> Result<()> {
        let config = AcademicConfig {
            institution: "IIIT Kottayam".to_string(),
            program: "B.Tech CSE".to_string(),
            batch: "2023".to_string(),
            grading_scale: GradingScale::Indian10,
            total_semesters: 8,
        };

        self.save_config("institution", &config.institution)?;
        self.save_config("program", &config.program)?;
        self.save_config("batch", &config.batch)?;
        self.save_config("total_semesters", &config.total_semesters.to_string())?;

        // Create default semesters
        let current_year = Utc::now().year();
        for sem in 1..=8 {
            let sem_id = format!("sem-{}", sem);
            let start_month = if sem % 2 == 1 { 7 } else { 1 };
            let end_month = if sem % 2 == 1 { 11 } else { 5 };
            let year_offset = (sem - 1) / 2;

            let start =
                NaiveDate::from_ymd_opt(current_year - 2 + year_offset, start_month, 1).unwrap();
            let end =
                NaiveDate::from_ymd_opt(current_year - 2 + year_offset, end_month, 30).unwrap();

            let is_current = sem == 5; // Assume 5th semester is current for 2023 batch

            self.conn.lock().unwrap().execute(
                "INSERT OR REPLACE INTO semesters (id, number, name, start_date, end_date, is_current)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    sem_id,
                    sem,
                    format!("Semester {}", sem),
                    start.to_string(),
                    end.to_string(),
                    is_current as i32,
                ],
            )?;
        }

        // Add default CSE courses for current semester
        let default_courses = vec![
            ("CS301", "Data Structures & Algorithms", 4, "theory"),
            ("CS302", "Operating Systems", 4, "theory"),
            ("CS303", "Database Management Systems", 3, "theory"),
            ("CS304", "Computer Networks", 3, "theory"),
            ("CS305", "Software Engineering", 3, "theory"),
            ("CS306", "DAA Lab", 1, "lab"),
            ("CS307", "OS Lab", 1, "lab"),
            ("CS308", "DBMS Lab", 1, "lab"),
        ];

        let conn = self.conn.lock().unwrap();
        for (code, name, credits, course_type) in &default_courses {
            let course_id = format!("sem-5-{}", code);
            conn.execute(
                "INSERT OR IGNORE INTO courses (id, semester_id, code, name, credits, type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![course_id, "sem-5", code, name, credits, course_type],
            )?;
        }

        Ok(())
    }

    pub fn current_semester(&self) -> Result<Option<Semester>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, number, name, start_date, end_date, is_current
             FROM semesters WHERE is_current = 1",
        )?;

        let mut rows = stmt.query_map([], |row| {
            Ok(Semester {
                id: row.get(0)?,
                number: row.get(1)?,
                name: row.get(2)?,
                start_date: NaiveDate::parse_from_str(&row.get::<_, String>(3)?, "%Y-%m-%d")
                    .unwrap_or_default(),
                end_date: NaiveDate::parse_from_str(&row.get::<_, String>(4)?, "%Y-%m-%d")
                    .unwrap_or_default(),
                is_current: row.get::<_, i32>(5)? != 0,
            })
        })?;

        match rows.next() {
            Some(Ok(sem)) => Ok(Some(sem)),
            _ => Ok(None),
        }
    }

    pub fn list_semesters(&self) -> Result<Vec<Semester>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, number, name, start_date, end_date, is_current FROM semesters ORDER BY number",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Semester {
                id: row.get(0)?,
                number: row.get(1)?,
                name: row.get(2)?,
                start_date: NaiveDate::parse_from_str(&row.get::<_, String>(3)?, "%Y-%m-%d")
                    .unwrap_or_default(),
                end_date: NaiveDate::parse_from_str(&row.get::<_, String>(4)?, "%Y-%m-%d")
                    .unwrap_or_default(),
                is_current: row.get::<_, i32>(5)? != 0,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn save_config(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_config(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM config WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(Ok(val)) => Ok(Some(val)),
            _ => Ok(None),
        }
    }
}
