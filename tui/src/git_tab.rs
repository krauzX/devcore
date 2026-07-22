use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use crate::theme;
use devcore_devtrack::{SkillAxis, MAX_LEVEL, XP_PER_LEVEL};

pub fn render_git_tab(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .spacing(1)
        .split(area);

    render_streak_stats(frame, chunks[0], app);
    render_skill_bars(frame, chunks[1], app);
}

fn render_streak_stats(frame: &mut Frame, area: Rect, app: &App) {
    let items = match &app.streak {
        Some(s) => vec![
            ListItem::new(Line::from(vec![
                Span::styled("Current Streak: ", Style::default().fg(theme::SUBTEXT)),
                Span::styled(
                    format!("{} days", s.current),
                    Style::default()
                        .fg(theme::GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("Longest Streak: ", Style::default().fg(theme::SUBTEXT)),
                Span::styled(
                    format!("{} days", s.longest),
                    Style::default().fg(theme::YELLOW),
                ),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("Total Days:     ", Style::default().fg(theme::SUBTEXT)),
                Span::styled(
                    format!("{}", s.total_days),
                    Style::default().fg(theme::TEAL),
                ),
            ])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::styled("Last Commit:    ", Style::default().fg(theme::SUBTEXT)),
                Span::styled(
                    s.last_commit_date
                        .map(|d: chrono::NaiveDate| d.to_string())
                        .unwrap_or_else(|| "none".to_string()),
                    Style::default().fg(theme::OVERLAY),
                ),
            ])),
        ],
        None => vec![ListItem::new(Line::from(Span::styled(
            "No git repository detected in project root",
            Style::default().fg(theme::OVERLAY),
        )))],
    };

    let list = List::new(items).block(
        Block::default()
            .title(" Streak Stats ")
            .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY))
            .style(Style::default().bg(theme::SURFACE)),
    );
    frame.render_widget(list, area);
}

fn render_skill_bars(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Skill Progression ")
        .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::OVERLAY))
        .style(Style::default().bg(theme::SURFACE));

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
        .spacing(1)
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
            .unwrap_or(devcore_devtrack::SkillProgress {
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

        let gauge_color = axis_gauge_color(axis);

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(Span::styled(
                        label,
                        Style::default().fg(theme::TEXT),
                    ))
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::OVERLAY)),
            )
            .gauge_style(
                Style::default()
                    .fg(gauge_color)
                    .bg(theme::BASE)
                    .add_modifier(Modifier::BOLD),
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

fn axis_gauge_color(axis: &SkillAxis) -> Color {
    match axis {
        SkillAxis::CommitHygiene => theme::BLUE,
        SkillAxis::Testing => theme::GREEN,
        SkillAxis::Documentation => theme::TEAL,
        SkillAxis::CodeReview => theme::MAUVE,
        SkillAxis::Architecture => theme::PEACH,
    }
}
