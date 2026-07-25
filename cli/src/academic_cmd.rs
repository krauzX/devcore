use std::path::Path;

use anyhow::Result;
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use devcore_academic::{Course, Deadline, GradeEntry, Semester, SemesterStore, score_to_grade};
use uuid::Uuid;

#[derive(Parser)]
pub struct AcademicCmd {
    #[command(subcommand)]
    pub action: AcademicAction,
}

#[derive(Subcommand)]
pub enum AcademicAction {
    /// Show academic dashboard summary
    Dashboard,
    /// Show current semester
    Current,
    /// List all semesters
    List,
    /// Add a new semester
    Add {
        name: String,
        number: String,
        start: String,
        end: String,
    },
    /// Set current semester
    Set {
        id: String,
    },
    /// Show upcoming deadlines with priority indicators
    Deadlines {
        #[arg(default_value_t = 30)]
        days: i64,
    },
    /// Add a deadline
    AddDeadline {
        title: String,
        date: String,
        #[arg(default_value = "")]
        notes: String,
    },
    /// Add a grade for a course
    Grade {
        #[arg(long)]
        course: String,
        #[arg(long)]
        exam: String,
        #[arg(long)]
        obtained: f64,
        #[arg(long)]
        total: f64,
    },
    /// Add a course to current semester
    Course {
        #[arg(long)]
        code: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        credits: i32,
    },
    /// Compute SGPA for current semester
    Sgpa,
    /// Compute overall CGPA across all semesters
    Cgpa,
}

pub fn run(cmd: AcademicCmd, project_root: &Path) -> Result<()> {
    match cmd.action {
        AcademicAction::Dashboard => run_dashboard(project_root),
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
            Ok(())
        }
        AcademicAction::List => {
            let store = SemesterStore::open(project_root)?;
            let semesters = store.list_semesters()?;
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
            Ok(())
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
            store.add_semester(&sem)?;
            println!("Added semester '{}' with id {}", sem.name, id);
            Ok(())
        }
        AcademicAction::Set { id } => {
            let store = SemesterStore::open(project_root)?;
            store.set_current_semester(&id)?;
            println!("Set current semester to {}", id);
            Ok(())
        }
        AcademicAction::Deadlines { days } => run_deadlines(project_root, days),
        AcademicAction::AddDeadline {
            title,
            date,
            notes,
        } => {
            let store = SemesterStore::open(project_root)?;
            let current = store
                .current_semester()
                .ok_or_else(|| anyhow::anyhow!("No current semester set. Use 'set' first."))?;
            let due_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid date '{}': {}", date, e))?;
            let conn = store.conn().map_err(|e| anyhow::anyhow!(e))?;
            let deadline = Deadline {
                id: Uuid::new_v4().to_string(),
                semester_id: current.id,
                title,
                description: notes,
                due_date,
                completed: false,
            };
            Deadline::add(&conn, &deadline)?;
            println!(
                "Added deadline '{}' due {}",
                deadline.title, deadline.due_date
            );
            Ok(())
        }
        AcademicAction::Grade {
            course: course_code,
            exam,
            obtained,
            total,
        } => run_grade(project_root, &course_code, &exam, obtained, total),
        AcademicAction::Course {
            code,
            name,
            credits,
        } => run_course(project_root, &code, &name, credits),
        AcademicAction::Sgpa => run_sgpa(project_root),
        AcademicAction::Cgpa => run_cgpa(project_root),
    }
}

fn run_dashboard(project_root: &Path) -> Result<()> {
    let store = SemesterStore::open(project_root)?;
    let conn = store.conn().map_err(|e| anyhow::anyhow!(e))?;

    println!("========================================");
    println!("          ACADEMIC DASHBOARD");
    println!("========================================");

    match store.current_semester() {
        Some(sem) => {
            println!();
            println!("  Semester : {} ({})", sem.name, sem.id);
            println!("  Period   : {} to {}", sem.start_date, sem.end_date);

            let course_count = Course::count_for_semester(&conn, &sem.id);
            let total_credits = Course::total_credits_for_semester(&conn, &sem.id);
            println!("  Courses  : {} ({} total credits)", course_count, total_credits);

            match GradeEntry::compute_sgpa(&conn, &sem.id) {
                Some(sgpa) => {
                    let bar_len: usize = 20;
                    let filled = ((sgpa / 10.0) * bar_len as f64) as usize;
                    let empty = bar_len.saturating_sub(filled);
                    println!(
                        "  SGPA     : [{}{}] {:.2}",
                        "█".repeat(filled),
                        "░".repeat(empty),
                        sgpa
                    );
                }
                None => println!("  SGPA     : -- (no grades yet)"),
            }

            let deadlines = Deadline::upcoming(&conn, 30).unwrap_or_default();
            if let Some(next) = deadlines.first() {
                let days_left = Deadline::days_until(next.due_date);
                let priority = Deadline::urgency_label(days_left);
                let label = if priority.is_empty() {
                    String::new()
                } else {
                    format!("[{}] ", priority)
                };
                println!(
                    "  Next     : {}{} ({}d left)",
                    label, next.title, days_left
                );
            } else {
                println!("  Next     : No upcoming deadlines");
            }

            println!();
        }
        None => {
            println!();
            println!("  No current semester set. Use 'devcore academic set <id>'");
            println!();
        }
    }

    match GradeEntry::compute_cgpa(&conn) {
        Some(cgpa) => {
            println!("  Overall CGPA: {:.2}", cgpa);
        }
        None => println!("  Overall CGPA: -- (no grades yet)"),
    }

    println!();
    println!("========================================");
    Ok(())
}

fn run_deadlines(project_root: &Path, days: i64) -> Result<()> {
    let store = SemesterStore::open(project_root)?;
    let conn = store.conn().map_err(|e| anyhow::anyhow!(e))?;
    let deadlines = Deadline::upcoming(&conn, days)?;

    if deadlines.is_empty() {
        println!("No upcoming deadlines in the next {} days.", days);
        return Ok(());
    }

    println!(
        "Upcoming deadlines (next {} days):",
        days
    );
    println!("{}", "-".repeat(60));

    for d in &deadlines {
        let days_left = Deadline::days_until(d.due_date);
        let priority = Deadline::urgency_label(days_left);

        let urgency_str = if days_left <= 0 {
            "\x1b[31m"  // red
        } else if days_left <= 1 {
            "\x1b[31m"  // red
        } else if days_left <= 3 {
            "\x1b[33m"  // yellow
        } else if days_left <= 7 {
            "\x1b[36m"  // cyan
        } else {
            "\x1b[32m"  // green
        };

        let reset = "\x1b[0m";

        let label = if priority.is_empty() {
            format!("     ")
        } else {
            format!("[{:<3}]", priority)
        };

        println!(
            "{}{}{} {}  due {} ({}d){}",
            urgency_str, label, reset, d.title, d.due_date, days_left, ""
        );
    }

    println!();
    println!("  {} deadline(s) found", deadlines.len());
    Ok(())
}

fn run_grade(
    project_root: &Path,
    course_code: &str,
    exam: &str,
    obtained: f64,
    total: f64,
) -> Result<()> {
    let store = SemesterStore::open(project_root)?;
    let current = store
        .current_semester()
        .ok_or_else(|| anyhow::anyhow!("No current semester set. Use 'set' first."))?;
    let conn = store.conn().map_err(|e| anyhow::anyhow!(e))?;

    let course = Course::find_by_code(&conn, course_code)
        .ok_or_else(|| anyhow::anyhow!("Course '{}' not found", course_code))?;

    let grade_str = score_to_grade(obtained, total);
    let points = devcore_academic::grade_to_points(grade_str);

    let semester_id = current.id.clone();
    let entry = devcore_academic::GradeEntry {
        id: Uuid::new_v4().to_string(),
        course_id: course.id.clone(),
        semester_id: semester_id.clone(),
        grade: grade_str.to_string(),
    };
    GradeEntry::add(&conn, &entry)?;

    println!("Grade added:");
    println!("  Course   : {} ({})", course.name, course.code);
    println!("  Exam     : {}", exam);
    println!(
        "  Score    : {:.1}/{:.1} ({:.1}%)",
        obtained,
        total,
        (obtained / total) * 100.0
    );
    println!("  Grade    : {} ({:.1} points)", grade_str, points);

    if let Some(sgpa) = GradeEntry::compute_sgpa(&conn, &semester_id) {
        println!("  Semester SGPA: {:.2}", sgpa);
    }

    Ok(())
}

fn run_course(project_root: &Path, code: &str, name: &str, credits: i32) -> Result<()> {
    let store = SemesterStore::open(project_root)?;
    let current = store
        .current_semester()
        .ok_or_else(|| anyhow::anyhow!("No current semester set. Use 'set' first."))?;
    let conn = store.conn().map_err(|e| anyhow::anyhow!(e))?;

    let course = Course {
        id: Uuid::new_v4().to_string(),
        semester_id: current.id.clone(),
        name: name.to_string(),
        code: code.to_string(),
        credits,
    };
    Course::add(&conn, &course)?;

    println!("Course added:");
    println!("  Code     : {}", course.code);
    println!("  Name     : {}", course.name);
    println!("  Credits  : {}", course.credits);
    println!("  Semester : {}", current.name);

    let count = Course::count_for_semester(&conn, &current.id);
    let total = Course::total_credits_for_semester(&conn, &current.id);
    println!();
    println!("  Total: {} courses ({} credits)", count, total);

    Ok(())
}

fn run_sgpa(project_root: &Path) -> Result<()> {
    let store = SemesterStore::open(project_root)?;
    let current = store
        .current_semester()
        .ok_or_else(|| anyhow::anyhow!("No current semester set."))?;
    let conn = store.conn().map_err(|e| anyhow::anyhow!(e))?;

    println!("SGPA for {}:", current.name);

    match GradeEntry::compute_sgpa(&conn, &current.id) {
        Some(sgpa) => {
            let bar_len: usize = 30;
            let filled = ((sgpa / 10.0) * bar_len as f64) as usize;
            let empty = bar_len.saturating_sub(filled);
            println!();
            println!(
                "  [{}{}] {:.2}/10.00",
                "█".repeat(filled),
                "░".repeat(empty),
                sgpa
            );
            println!();
        }
        None => println!("  No grades recorded for this semester yet."),
    }

    Ok(())
}

fn run_cgpa(project_root: &Path) -> Result<()> {
    let store = SemesterStore::open(project_root)?;
    let conn = store.conn().map_err(|e| anyhow::anyhow!(e))?;

    println!("Overall CGPA (all semesters):");

    match GradeEntry::compute_cgpa(&conn) {
        Some(cgpa) => {
            let bar_len: usize = 30;
            let filled = ((cgpa / 10.0) * bar_len as f64) as usize;
            let empty = bar_len.saturating_sub(filled);
            println!();
            println!(
                "  [{}{}] {:.2}/10.00",
                "█".repeat(filled),
                "░".repeat(empty),
                cgpa
            );
            println!();
        }
        None => println!("  No grades recorded yet."),
    }

    let semesters = store.list_semesters()?;
    if !semesters.is_empty() {
        println!("  Per-semester breakdown:");
        for sem in &semesters {
            if let Some(sgpa) = GradeEntry::compute_sgpa(&conn, &sem.id) {
                let marker = if sem.is_current { " *" } else { "" };
                println!("    {}: {:.2}{}", sem.name, sgpa, marker);
            }
        }
    }

    Ok(())
}
