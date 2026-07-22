use std::path::{Path, PathBuf};
use std::sync::mpsc;

use anyhow::Result;
use devcore_academic::{Deadline, SemesterStore};

use muda::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::TrayIconBuilder;
use tao::event_loop::{ControlFlow, EventLoopBuilder};


const ID_SHOW_DEADLINES: &str = "show_deadlines";
const ID_CURRENT_SEMESTER: &str = "current_semester";
const ID_LIST_SEMESTERS: &str = "list_semesters";
const ID_QUIT: &str = "quit";

pub struct TrayApp {
    project_root: PathBuf,
}

impl TrayApp {
    pub fn new(project_root: &Path) -> Result<Self> {
        Ok(Self {
            project_root: project_root.to_path_buf(),
        })
    }

    pub fn run(&self) -> Result<()> {
        let project_root = self.project_root.clone();

        let menu = Menu::with_items(&[
            &MenuItem::with_id(ID_CURRENT_SEMESTER, "Current Semester", true, None),
            &MenuItem::with_id(ID_LIST_SEMESTERS, "List Semesters", true, None),
            &MenuItem::with_id(ID_SHOW_DEADLINES, "Upcoming Deadlines (7 days)", true, None),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id(ID_QUIT, "Quit", true, None),
        ])?;

        let _tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("DevCore")
            .build()?;

        let (tx, rx) = mpsc::channel::<String>();

        muda::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
            let _ = tx.send(event.id.0);
        }));

        let event_loop = EventLoopBuilder::new().build();

        event_loop.run(move |_event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            if let Ok(id) = rx.try_recv() {
                match id.as_str() {
                    ID_CURRENT_SEMESTER => {
                        let _ = show_current_semester(&project_root);
                    }
                    ID_LIST_SEMESTERS => {
                        let _ = list_semesters(&project_root);
                    }
                    ID_SHOW_DEADLINES => {
                        let _ = show_deadlines(&project_root, 7);
                    }
                    ID_QUIT => {
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
        });
    }
}

fn show_current_semester(project_root: &Path) -> Result<()> {
    let store = SemesterStore::open(project_root)?;
    match store.current_semester() {
        Some(sem) => {
            println!("Current: {} ({} to {})", sem.name, sem.start_date, sem.end_date);
        }
        None => {
            println!("No current semester set.");
        }
    }
    Ok(())
}

fn list_semesters(project_root: &Path) -> Result<()> {
    let store = SemesterStore::open(project_root)?;
    let semesters = store.list_semesters().unwrap_or_default();
    for sem in &semesters {
        let marker = if sem.is_current { " *" } else { "" };
        println!(
            "[{}] {} — {} to {}{}",
            &sem.id.get(..8).unwrap_or(&sem.id), sem.name, sem.start_date, sem.end_date, marker
        );
    }
    Ok(())
}

fn show_deadlines(project_root: &Path, days: i64) -> Result<()> {
    let store = SemesterStore::open(project_root)?;
    let conn = store.conn();
    let deadlines = Deadline::upcoming(&conn, days).unwrap_or_default();
    if deadlines.is_empty() {
        println!("No upcoming deadlines in the next {} days.", days);
    } else {
        for d in &deadlines {
            println!("{} — due {}", d.title, d.due_date);
        }
    }
    Ok(())
}
