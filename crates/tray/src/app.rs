use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;

use anyhow::Result;
use devcore_academic::{Deadline, GradeEntry, SemesterStore};
use devcore_devtrack::compute_streak;
use muda::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::TrayIconBuilder;
use tao::event_loop::{ControlFlow, EventLoopBuilder};

const ID_VIEW_DASHBOARD: &str = "view_dashboard";
const ID_QUIT: &str = "quit";

pub struct TrayApp {
    project_root: PathBuf,
}

struct DailyStats {
    deadlines_today: usize,
    next_deadline: Option<String>,
    streak_days: u32,
    sgpa: Option<f64>,
}

impl TrayApp {
    pub fn new(project_root: &Path) -> Result<Self> {
        Ok(Self {
            project_root: project_root.to_path_buf(),
        })
    }

    pub fn run(&self) -> Result<()> {
        let stats = self.load_stats();
        let tooltip = self.build_tooltip(&stats);
        let menu = self.build_menu(&stats)?;

        let _tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(&tooltip)
            .build()?;

        let (tx, rx) = mpsc::channel::<String>();
        muda::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
            let _ = tx.send(event.id.0);
        }));

        let project_root = self.project_root.clone();
        let event_loop = EventLoopBuilder::new().build();
        event_loop.run(move |_event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            if let Ok(id) = rx.try_recv() {
                match id.as_str() {
                    ID_VIEW_DASHBOARD => {
                        let _ = launch_tui(&project_root);
                    }
                    ID_QUIT => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
        });
    }

    fn load_stats(&self) -> DailyStats {
        let (deadlines_today, next_deadline, sgpa) =
            SemesterStore::open(&self.project_root)
                .map_err(|e| eprintln!("tray: {}", e))
                .ok()
                .and_then(|s| {
                    let c = s.conn().ok()?;
                    let deadlines = Deadline::upcoming(&c, 1).unwrap_or_default();
                    let next_name = deadlines.first().map(|d| {
                        let days = Deadline::days_until(d.due_date);
                        let urg = Deadline::urgency_label(days);
                        if urg.is_empty() { d.title.clone() } else { format!("{} {}", d.title, urg) }
                    });
                    let sgpa = s.current_semester().and_then(|sem| GradeEntry::compute_sgpa(&c, &sem.id));
                    Some((deadlines.len(), next_name, sgpa))
                })
                .unwrap_or((0, None, None));

        let streak_days = compute_streak(&self.project_root)
            .map(|s| s.current)
            .map_err(|e| eprintln!("tray: {}", e))
            .unwrap_or(0);

        DailyStats { deadlines_today, next_deadline, streak_days, sgpa }
    }

    fn build_tooltip(&self, stats: &DailyStats) -> String {
        let sgpa_str = match stats.sgpa {
            Some(g) => format!("{:.2}", g),
            None => "--".into(),
        };
        format!(
            "DevCore | {} due today | {}d streak | SGPA {}",
            stats.deadlines_today, stats.streak_days, sgpa_str
        )
    }

    fn build_menu(&self, stats: &DailyStats) -> Result<Menu> {
        let next_label = match &stats.next_deadline {
            Some(name) => format!("Next: {}", name),
            None => "No deadlines today".into(),
        };
        let deadline_label = format!("{} due today", stats.deadlines_today);
        let streak_label = format!("Streak: {} days", stats.streak_days);
        let sgpa_label = match stats.sgpa {
            Some(g) => format!("SGPA: {:.2}", g),
            None => "SGPA: --".into(),
        };

        let menu = Menu::with_items(&[
            &MenuItem::with_id("header", "Daily Dashboard", false, None),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id("deadlines", deadline_label, false, None),
            &MenuItem::with_id("next_deadline", next_label, false, None),
            &MenuItem::with_id("streak", streak_label, false, None),
            &MenuItem::with_id("sgpa", sgpa_label, false, None),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id(ID_VIEW_DASHBOARD, "Open Dashboard", true, None),
            &PredefinedMenuItem::separator(),
            &MenuItem::with_id(ID_QUIT, "Quit", true, None),
        ])?;
        Ok(menu)
    }
}

fn launch_tui(project_root: &Path) -> Result<()> {
    let exe = std::env::current_exe()?;
    let dir = exe.parent().unwrap_or(project_root);
    let binary = if cfg!(windows) { "devcore.exe" } else { "devcore" };
    Command::new(dir.join(binary))
        .arg("tui")
        .current_dir(project_root)
        .spawn()?;
    Ok(())
}
