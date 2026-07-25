use chrono::NaiveDate;
use rusqlite::{params, Connection};

#[derive(Debug, Clone)]
pub struct Deadline {
    pub id: String,
    pub semester_id: String,
    pub title: String,
    pub description: String,
    pub due_date: NaiveDate,
    pub completed: bool,
}

impl Deadline {
    pub fn upcoming(conn: &Connection, days: i64) -> Result<Vec<Deadline>, rusqlite::Error> {
        let today = chrono::Local::now().naive_local().date();
        let cutoff = today + chrono::Duration::days(days);
        let mut stmt = conn.prepare(
            "SELECT id, semester_id, title, description, due_date, completed \
             FROM deadlines \
             WHERE completed = 0 AND due_date BETWEEN ?1 AND ?2 \
             ORDER BY due_date",
        )?;
        let rows = stmt.query_map(params![today.to_string(), cutoff.to_string()], |row| {
            Ok(Deadline {
                id: row.get(0)?,
                semester_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                due_date: row.get::<_, String>(4)?.parse().unwrap_or_default(),
                completed: row.get::<_, i32>(5)? != 0,
            })
        })?;
        let deadlines = rows.filter_map(|r| r.ok()).collect();
        Ok(deadlines)
    }

    pub fn add(conn: &Connection, deadline: &Deadline) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO deadlines (id, semester_id, title, description, due_date, completed) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                deadline.id,
                deadline.semester_id,
                deadline.title,
                deadline.description,
                deadline.due_date.to_string(),
                deadline.completed as i32,
            ],
        )?;
        Ok(())
    }

    pub fn complete(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE deadlines SET completed = 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn list_all(conn: &Connection, semester_id: &str) -> Result<Vec<Deadline>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, semester_id, title, description, due_date, completed \
             FROM deadlines \
             WHERE semester_id = ?1 \
             ORDER BY due_date",
        )?;
        let rows = stmt.query_map(params![semester_id], |row| {
            Ok(Deadline {
                id: row.get(0)?,
                semester_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                due_date: row.get::<_, String>(4)?.parse().unwrap_or_default(),
                completed: row.get::<_, i32>(5)? != 0,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn days_until(due_date: NaiveDate) -> i64 {
        let today = chrono::Local::now().naive_local().date();
        (due_date - today).num_days()
    }

    pub fn urgency_label(days_left: i64) -> &'static str {
        if days_left <= 0 {
            "OVERDUE"
        } else if days_left <= 1 {
            "!!!"
        } else if days_left <= 3 {
            "!!"
        } else if days_left <= 7 {
            "!"
        } else {
            ""
        }
    }
}
