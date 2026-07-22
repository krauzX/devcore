use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use devcore_gitforge::{SkillAxis, MAX_LEVEL, XP_PER_LEVEL};

pub fn render_git_tab(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_streak_stats(frame, chunks[0], app);
    render_skill_bars(frame, chunks[1], app);
}

fn render_streak_stats(frame: &mut Frame, area: Rect, app: &App) {
    let items = match &app.streak {
        Some(s) => vec![
            ListItem::new(Line::from(vec![
                Span::styled("Current Streak: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{} days", s.current),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("Longest Streak: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{} days", s.longest),
                    Style::default().fg(Color::Yellow),
                ),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("Total Days:     ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("{}", s.total_days),
                    Style::default().fg(Color::Cyan),
                ),
            ])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::styled("Last Commit:    ", Style::default().fg(Color::White)),
                Span::styled(
                    s.last_commit_date
                        .map(|d: chrono::NaiveDate| d.to_string())
                        .unwrap_or_else(|| "none".to_string()),
                    Style::default().fg(Color::DarkGray),
                ),
            ])),
        ],
        None => vec![ListItem::new(Line::from(Span::styled(
            "No git repository detected in project root",
            Style::default().fg(Color::DarkGray),
        )))],
    };

    let list = List::new(items).block(
        Block::default()
            .title(" Streak Stats ")
            .borders(Borders::ALL),
    );
    frame.render_widget(list, area);
}

fn render_skill_bars(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Skill Progression ")
        .borders(Borders::ALL);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let axes = SkillAxis::all();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            axes.iter()
                .map(|_| Constraint::Length(3))
                .collect::<Vec<_>>(),
        )
        .split(inner);

    for (i, axis) in axes.iter().enumerate() {
        if i >= chunks.len() {
            break;
        }

        let progress = app
            .skill_progress
            .iter()
            .find(|sp| sp.axis == *axis)
            .cloned()
            .unwrap_or(devcore_gitforge::SkillProgress {
                axis: *axis,
                xp: 0,
                level: 0,
            });

        let max_xp = MAX_LEVEL * XP_PER_LEVEL;
        let ratio = (progress.xp as f64 / max_xp as f64).min(1.0);

        let label = format!(
            "{} (Lv {} - {} XP)",
            axis_name(axis),
            progress.level,
            progress.xp
        );

        let gauge = Gauge::default()
            .block(Block::default().title(label))
            .gauge_style(
                Style::default()
                    .fg(Color::Cyan)
                    .bg(Color::DarkGray),
            )
            .ratio(ratio);

        frame.render_widget(gauge, chunks[i]);
    }
}

fn axis_name(axis: &SkillAxis) -> &'static str {
    match axis {
        SkillAxis::CommitHygiene => "Commit Hygiene",
        SkillAxis::Testing => "Testing",
        SkillAxis::Documentation => "Documentation",
        SkillAxis::CodeReview => "Code Review",
        SkillAxis::Architecture => "Architecture",
    }
}
