use std::path::Path;

use anyhow::Result;
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use devcore_academic::{Deadline, Semester, SemesterStore};
use uuid::Uuid;

#[derive(Parser)]
pub struct AcademicCmd {
    #[command(subcommand)]
    pub action: AcademicAction,
}

#[derive(Subcommand)]
pub enum AcademicAction {
    Current,
    List,
    Add {
        name: String,
        number: String,
        start: String,
        end: String,
    },
    Set {
        id: String,
    },
    Deadlines {
        #[arg(default_value_t = 30)]
        days: i64,
    },
    AddDeadline {
        title: String,
        date: String,
        #[arg(default_value = "")]
        priority: String,
        #[arg(default_value = "")]
        notes: String,
    },
}

pub fn run(cmd: AcademicCmd, project_root: &Path) -> Result<()> {
    match cmd.action {
        AcademicAction::Current => {
            let store = SemesterStore::open(project_root)?;
            match store.current_semester() {
                Some(sem) => {
                    println!("Current semester: {} ({})", sem.name, sem.id);
                    println!("  Start: {}", sem.start_date);
                    println!("  End:   {}", sem.end_date);
                }
                None => println!("No current semester set."),
            }
        }
        AcademicAction::List => {
            let store = SemesterStore::open(project_root)?;
            let semesters = store.list_semesters();
            if semesters.is_empty() {
                println!("No semesters found.");
            } else {
                for sem in &semesters {
                    let marker = if sem.is_current { " *" } else { "" };
                    println!(
                        "[{}] {} — {} to {}{}",
                        sem.id, sem.name, sem.start_date, sem.end_date, marker
                    );
                }
            }
        }
        AcademicAction::Add {
            name,
            number,
            start,
            end,
        } => {
            let store = SemesterStore::open(project_root)?;
            let start_date = NaiveDate::parse_from_str(&start, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid start date '{}': {}", start, e))?;
            let end_date = NaiveDate::parse_from_str(&end, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid end date '{}': {}", end, e))?;
            let id = Uuid::new_v4().to_string();
            let sem = Semester {
                id: id.clone(),
                name: format!("{} {}", name, number),
                start_date,
                end_date,
                is_current: false,
            };
            store.add_semester(&sem);
            println!("Added semester '{}' with id {}", sem.name, id);
        }
        AcademicAction::Set { id } => {
            let store = SemesterStore::open(project_root)?;
            store.set_current_semester(&id);
            println!("Set current semester to {}", id);
        }
        AcademicAction::Deadlines { days } => {
            let store = SemesterStore::open(project_root)?;
            let conn = store.conn();
            let deadlines = Deadline::upcoming(&conn, days);
            if deadlines.is_empty() {
                println!("No upcoming deadlines in the next {} days.", days);
            } else {
                for d in &deadlines {
                    println!(
                        "[{}] {} — due {}",
                        &d.id[..8],
                        d.title,
                        d.due_date
                    );
                }
            }
        }
        AcademicAction::AddDeadline {
            title,
            date,
            priority: _,
            notes,
        } => {
            let store = SemesterStore::open(project_root)?;
            let current = store.current_semester()
                .ok_or_else(|| anyhow::anyhow!("No current semester set. Use 'set' first."))?;
            let due_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid date '{}': {}", date, e))?;
            let conn = store.conn();
            let deadline = Deadline {
                id: Uuid::new_v4().to_string(),
                semester_id: current.id,
                title,
                description: notes,
                due_date,
                completed: false,
            };
            Deadline::add(&conn, &deadline);
            println!("Added deadline '{}' due {}", deadline.title, deadline.due_date);
        }
    }
    Ok(())
}
