use crate::tui::dashboard;
use anyhow::Result;
use chrono::{Duration, Timelike, Utc};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use devcore_core::*;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration as StdDuration, Instant};

#[derive(Clone, PartialEq)]
pub enum Tab {
    Dashboard,
    Sessions,
    Heatmap,
    Suggestions,
}

pub struct App {
    pub store: Store,
    pub git: Option<GitAnalyzer>,
    pub should_quit: bool,
    pub active_tab: Tab,
    pub session_start: Option<Instant>,
    pub session_elapsed: StdDuration,
    pub events_today: Vec<WorkflowEvent>,
    pub commits_today: Vec<CommitInfo>,
    pub hourly_activity: HashMap<u32, u32>,
    pub categories: HashMap<String, f64>,
    pub total_minutes: f64,
}

impl App {
    pub fn new(project_root: &Path) -> Result<Self> {
        let store = Store::open(project_root)?;
        let git = GitAnalyzer::open(project_root).ok();

        let mut app = Self {
            store,
            git,
            should_quit: false,
            active_tab: Tab::Dashboard,
            session_start: None,
            session_elapsed: StdDuration::ZERO,
            events_today: Vec::new(),
            commits_today: Vec::new(),
            hourly_activity: HashMap::new(),
            categories: HashMap::new(),
            total_minutes: 0.0,
        };

        app.refresh_data()?;
        Ok(app)
    }

    fn refresh_data(&mut self) -> Result<()> {
        let since = Utc::now() - Duration::hours(24);
        self.events_today = self.store.events_since(since)?;

        if let Some(git) = &self.git {
            self.commits_today = git.commits_since(since, 100).unwrap_or_default();
        }

        self.hourly_activity.clear();
        self.categories.clear();
        self.total_minutes = 0.0;

        for event in &self.events_today {
            let hour = event.timestamp.hour();
            let mins = event
                .details
                .get("minutes")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            *self.hourly_activity.entry(hour).or_insert(0) += mins as u32;
            let cat = event
                .details
                .get("category")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            *self.categories.entry(cat).or_insert(0.0) += mins;
            self.total_minutes += mins;
        }

        for commit in &self.commits_today {
            let cat = if commit.is_ai_generated {
                "ai_coding"
            } else {
                "human_coding"
            };
            *self.categories.entry(cat.to_string()).or_insert(0.0) += 20.0;
            self.total_minutes += 20.0;
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

        let tick_rate = StdDuration::from_millis(250);
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|f| self.draw(f))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(StdDuration::ZERO);

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key.code);
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.on_tick();
                last_tick = Instant::now();
            }

            if self.should_quit {
                break;
            }
        }

        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Tab => {
                self.active_tab = match self.active_tab {
                    Tab::Dashboard => Tab::Sessions,
                    Tab::Sessions => Tab::Heatmap,
                    Tab::Heatmap => Tab::Suggestions,
                    Tab::Suggestions => Tab::Dashboard,
                };
            }
            KeyCode::Char('1') => self.active_tab = Tab::Dashboard,
            KeyCode::Char('2') => self.active_tab = Tab::Sessions,
            KeyCode::Char('3') => self.active_tab = Tab::Heatmap,
            KeyCode::Char('4') => self.active_tab = Tab::Suggestions,
            KeyCode::Char('s') => {
                if self.session_start.is_some() {
                    self.stop_session();
                } else {
                    self.start_session();
                }
            }
            KeyCode::Char('r') => {
                let _ = self.refresh_data();
            }
            _ => {}
        }
    }

    fn on_tick(&mut self) {
        if let Some(start) = self.session_start {
            self.session_elapsed = start.elapsed();
        }
    }

    fn start_session(&mut self) {
        self.session_start = Some(Instant::now());
        self.session_elapsed = StdDuration::ZERO;
    }

    fn stop_session(&mut self) {
        if let Some(start) = self.session_start {
            let elapsed = start.elapsed();
            let mins = (elapsed.as_secs() / 60) as u32;
            if mins > 0 {
                let details = serde_json::json!({
                    "minutes": mins,
                    "description": "TUI session",
                    "category": "coding",
                });
                let event = WorkflowEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    event_type: EventType::FileEdit,
                    details,
                };
                let _ = self.store.save_event(&event);
                let _ = self.refresh_data();
            }
        }
        self.session_start = None;
        self.session_elapsed = StdDuration::ZERO;
    }

    fn draw(&self, f: &mut Frame) {
        let area = f.area();

        let header = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        self.render_header(f, header[0]);
        self.render_tab_content(f, header[1]);
        self.render_footer(f, header[2]);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let session_status = if self.session_start.is_some() {
            "s:stop".green().to_string()
        } else {
            "s:start".yellow().to_string()
        };
        let session_label = format!("{}  ", session_status);

        let footer = Line::from(vec![
            Span::styled(
                " q",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(":quit  "),
            Span::styled(
                "Tab",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(":next  "),
            Span::styled(
                "1-4",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(":jump  "),
            Span::raw(&session_label),
            Span::styled(
                "r",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(":refresh"),
        ]);

        let footer_block = Paragraph::new(footer).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        f.render_widget(footer_block, area);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let tabs = vec![" Dashboard ", " Sessions ", " Heatmap ", " Suggestions "];

        let selected = match self.active_tab {
            Tab::Dashboard => 0,
            Tab::Sessions => 1,
            Tab::Heatmap => 2,
            Tab::Suggestions => 3,
        };

        let tabs = Tabs::new(tabs)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" DevPulse — Developer Workflow Analyzer ")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .select(selected)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(tabs, area);
    }

    fn render_tab_content(&self, f: &mut Frame, area: Rect) {
        match self.active_tab {
            Tab::Dashboard => dashboard::render(f, area, self),
            Tab::Sessions => self.render_sessions(f, area),
            Tab::Heatmap => self.render_heatmap(f, area),
            Tab::Suggestions => self.render_suggestions(f, area),
        }
    }

    fn render_sessions(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(area);

        let session_status = if let Some(start) = self.session_start {
            let elapsed = start.elapsed();
            let mins = elapsed.as_secs() / 60;
            let secs = elapsed.as_secs() % 60;
            format!("ACTIVE — {:02}:{:02}", mins, secs)
        } else {
            "INACTIVE — press 's' to start".to_string()
        };

        let session_block = Paragraph::new(session_status)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Current Session ")
                    .border_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(session_block, chunks[0]);

        let recent: Vec<ListItem> = self
            .events_today
            .iter()
            .rev()
            .take(20)
            .map(|e| {
                let cat = e
                    .details
                    .get("category")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let mins = e
                    .details
                    .get("minutes")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let time = e.timestamp.format("%H:%M").to_string();
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{:>5} ", time),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(format!("{:<12}", cat), Style::default().fg(Color::Cyan)),
                    Span::raw(format!("{:.0}m", mins)),
                ]))
            })
            .collect();

        let list = List::new(recent).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Recent Events "),
        );
        f.render_widget(list, chunks[1]);
    }

    fn render_heatmap(&self, f: &mut Frame, area: Rect) {
        let mut rows = Vec::new();

        for hour in 0..24 {
            let mins = self.hourly_activity.get(&hour).copied().unwrap_or(0);
            let bar_len = if mins > 0 {
                let max = self.hourly_activity.values().copied().max().unwrap_or(1);
                ((mins as f64 / max as f64) * 30.0) as usize
            } else {
                0
            };

            let intensity = if mins == 0 {
                Color::DarkGray
            } else if mins > 30 {
                Color::Red
            } else if mins > 15 {
                Color::Yellow
            } else {
                Color::Green
            };

            let bar: String = "█".repeat(bar_len);
            rows.push(Row::new(vec![
                Cell::from(format!("{:>2}:00", hour)).style(Style::default().fg(Color::DarkGray)),
                Cell::from(format!("{:>4}m", mins)),
                Cell::from(bar).style(Style::default().fg(intensity)),
                Cell::from(if mins > 0 { "●" } else { " " }).style(Style::default().fg(
                    if mins > 0 {
                        Color::Green
                    } else {
                        Color::DarkGray
                    },
                )),
            ]));
        }

        let table = Table::new(
            rows,
            [
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(0),
                Constraint::Length(2),
            ],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Activity Heatmap (24h) ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .header(
            Row::new(vec!["Hour", "Mins", "Activity", ""]).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        );

        f.render_widget(table, area);
    }

    fn render_suggestions(&self, f: &mut Frame, area: Rect) {
        let mut suggestions = Vec::new();

        if self.total_minutes == 0.0 {
            suggestions.push(ListItem::new(Line::from(Span::styled(
                "No activity recorded today. Press 's' to start a session.",
                Style::default().fg(Color::Yellow),
            ))));
        } else {
            if let Some((cat, mins)) = self
                .categories
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            {
                let pct = (mins / self.total_minutes) * 100.0;
                if pct > 50.0 {
                    suggestions.push(ListItem::new(Line::from(vec![
                        Span::styled("  ⚠ ", Style::default().fg(Color::Red)),
                        Span::styled(
                            format!("{} takes {:.0}% of your time", cat, pct),
                            Style::default().fg(Color::Red),
                        ),
                    ])));
                    suggestions.push(ListItem::new(Line::from(Span::raw(
                        "    Consider breaking this into smaller chunks",
                    ))));
                }
            }

            let ai_ratio = self
                .categories
                .get("ai_coding")
                .map(|ai| (ai / self.total_minutes) * 100.0)
                .unwrap_or(0.0);

            if ai_ratio > 70.0 {
                suggestions.push(ListItem::new(Line::from(vec![
                    Span::styled("  ⚠ ", Style::default().fg(Color::Red)),
                    Span::raw(format!(
                        "AI usage at {:.0}% — review quality may suffer",
                        ai_ratio
                    )),
                ])));
            } else if ai_ratio > 0.0 && ai_ratio < 30.0 {
                suggestions.push(ListItem::new(Line::from(vec![
                    Span::styled("  ✓ ", Style::default().fg(Color::Green)),
                    Span::raw("Balanced AI usage"),
                ])));
            }

            let commit_count = self.commits_today.len();
            if commit_count > 15 {
                suggestions.push(ListItem::new(Line::from(vec![
                    Span::styled("  💡 ", Style::default().fg(Color::Cyan)),
                    Span::raw(format!(
                        "{} commits today — consider batching related changes",
                        commit_count
                    )),
                ])));
            }

            let large_commits: Vec<_> = self
                .commits_today
                .iter()
                .filter(|c| (c.insertions + c.deletions) > 200)
                .collect();
            if !large_commits.is_empty() {
                suggestions.push(ListItem::new(Line::from(vec![
                    Span::styled("  ⚠ ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!(
                        "{} large commits (>200 changes)",
                        large_commits.len()
                    )),
                ])));
                suggestions.push(ListItem::new(Line::from(Span::raw(
                    "    Break into smaller pieces for easier review",
                ))));
            }

            if suggestions.is_empty() {
                suggestions.push(ListItem::new(Line::from(Span::styled(
                    "  ✓ Looking good! Keep it up.",
                    Style::default().fg(Color::Green),
                ))));
            }
        }

        let list = List::new(suggestions).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Productivity Suggestions ")
                .border_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(list, area);
    }
}
