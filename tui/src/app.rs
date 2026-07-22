use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use devcore_academic::{Deadline, GradeEntry, SemesterStore};
use devcore_challenges::{ChallengeEngine, ProblemPack};
use devcore_core::{DevCoreConfig, Store};
use devcore_devtrack::{SkillProgress, Streak};
use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::academic_tab::render_academic_tab;
use crate::challenges_tab::render_challenges_tab;
use crate::dashboard::render_dashboard;
use crate::git_tab::render_git_tab;
use crate::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Dashboard,
    Academic,
    Git,
    Challenges,
}

impl Tab {
    fn all() -> &'static [Tab] {
        &[Tab::Dashboard, Tab::Academic, Tab::Git, Tab::Challenges]
    }

    fn title(&self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Academic => "Academic",
            Tab::Git => "Git",
            Tab::Challenges => "Challenges",
        }
    }

    fn key(&self) -> char {
        match self {
            Tab::Dashboard => '1',
            Tab::Academic => '2',
            Tab::Git => '3',
            Tab::Challenges => '4',
        }
    }
}

pub struct App {
    should_quit: bool,
    active_tab: Tab,
    pub(crate) config: DevCoreConfig,
    pub(crate) semesters: Vec<devcore_academic::Semester>,
    pub(crate) current_semester: Option<devcore_academic::Semester>,
    pub(crate) upcoming_deadlines: Vec<Deadline>,
    pub(crate) sgpa: Option<f64>,
    pub(crate) streak: Option<Streak>,
    pub(crate) skill_progress: Vec<SkillProgress>,
    pub(crate) packs: Vec<ProblemPack>,
    pub(crate) installed_count: usize,
    pub(crate) solved_count: usize,
}

impl App {
    pub fn new(project_root: &Path) -> Result<Self> {
        let config = DevCoreConfig::load(project_root)?;

        let academic_store = SemesterStore::open(project_root)?;
        let semesters = academic_store.list_semesters().unwrap_or_default();
        let current_semester = academic_store.current_semester();
        let sgpa = current_semester.as_ref().and_then(|s| {
            let conn = academic_store.conn().ok()?;
            GradeEntry::compute_sgpa(&conn, &s.id)
        });
        let upcoming_deadlines = current_semester.as_ref().and_then(|_| {
            let conn = academic_store.conn().ok()?;
            Some(Deadline::upcoming(&conn, 30).unwrap_or_default())
        }).unwrap_or_default();

        let streak = devcore_devtrack::compute_streak(project_root).ok();

        let core_store = Store::open(project_root)?;
        let skill_progress = {
            let conn = core_store.conn()?;
            devcore_devtrack::init_skill_schema(&conn).ok();
            devcore_devtrack::get_progress(&conn).unwrap_or_default()
        };

        let engine = ChallengeEngine::new(project_root);
        let packs = engine.list_available().to_vec();
        let installed_count = engine.list_installed().len();
        let solved_count = {
            let conn = core_store.conn()?;
            devcore_challenges::progress::init_challenge_schema(&conn).ok();
            conn.query_row(
                "SELECT COUNT(*) FROM challenge_progress WHERE solved = 1",
                [],
                |row| row.get::<_, usize>(0),
            )
            .unwrap_or(0)
        };

        Ok(Self {
            should_quit: false,
            active_tab: Tab::Dashboard,
            config,
            semesters,
            current_semester,
            upcoming_deadlines,
            sgpa,
            streak,
            skill_progress,
            packs,
            installed_count,
            solved_count,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let exit_status = self.event_loop();
        ratatui::restore();
        exit_status
    }

    fn event_loop(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                self.should_quit = true;
                            }
                            KeyCode::Tab => {
                                let tabs = Tab::all();
                                let idx = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
                                self.active_tab = tabs[(idx + 1) % tabs.len()];
                            }
                            KeyCode::Char('1') => self.active_tab = Tab::Dashboard,
                            KeyCode::Char('2') => self.active_tab = Tab::Academic,
                            KeyCode::Char('3') => self.active_tab = Tab::Git,
                            KeyCode::Char('4') => self.active_tab = Tab::Challenges,
                            _ => {}
                        }
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::MAUVE))
            .title(Span::styled(
                " DevCore TUI ",
                Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(theme::BASE));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
            .split(inner);

        let spans: Vec<Span> = Tab::all()
            .iter()
            .map(|tab| {
                let style = if *tab == self.active_tab {
                    Style::default()
                        .fg(theme::YELLOW)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme::SUBTEXT)
                };
                Span::styled(
                    format!(" {} {} ", tab.key(), tab.title()),
                    style,
                )
            })
            .collect();
        let tab_bar = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        frame.render_widget(tab_bar, chunks[0]);

        match self.active_tab {
            Tab::Dashboard => render_dashboard(frame, chunks[1], self),
            Tab::Academic => render_academic_tab(frame, chunks[1], self),
            Tab::Git => render_git_tab(frame, chunks[1], self),
            Tab::Challenges => render_challenges_tab(frame, chunks[1], self),
        }

        let status_bar = Line::from(vec![
            Span::styled(" 1-4 ", Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD)),
            Span::styled("Tab ", Style::default().fg(theme::SUBTEXT)),
            Span::styled("│", Style::default().fg(theme::OVERLAY)),
            Span::styled(" q ", Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)),
            Span::styled("Quit ", Style::default().fg(theme::SUBTEXT)),
            Span::styled("│", Style::default().fg(theme::OVERLAY)),
            Span::styled(" Tab ", Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD)),
            Span::styled("Next ", Style::default().fg(theme::SUBTEXT)),
        ])
        .style(Style::default().bg(theme::SURFACE));
        frame.render_widget(status_bar, chunks[2]);
    }
}
