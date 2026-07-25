use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use crate::theme;
use crate::widgets;
use devcore_devtrack::{SkillAxis, MAX_LEVEL, XP_PER_LEVEL};

pub fn render_git_tab(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .spacing(1)
        .split(area);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .spacing(1)
        .split(chunks[0]);

    render_streak_panel(frame, left[0], app);
    render_repo_analysis(frame, left[1], app);
    render_skills_panel(frame, chunks[1], app);
}

fn render_streak_panel(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Streak Stats ")
        .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::OVERLAY))
        .style(Style::default().bg(theme::SURFACE));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if let Some(s) = &app.streak {
        let c = Layout::default().direction(Direction::Vertical)
            .constraints([Constraint::Length(1); 5]).split(inner);
        frame.render_widget(widgets::stat_row("Current Streak: ", &format!("{} days", s.current), theme::GREEN), c[0]);
        frame.render_widget(widgets::stat_row("Longest Streak: ", &format!("{} days", s.longest), theme::YELLOW), c[1]);
        frame.render_widget(widgets::stat_row("Total Days:     ", &s.total_days.to_string(), theme::TEAL), c[2]);
        let last = s.last_commit_date.map(|d| d.to_string()).unwrap_or_else(|| "none".to_string());
        let last_color = if s.last_commit_date.is_some() { theme::SUBTEXT } else { theme::OVERLAY };
        frame.render_widget(widgets::stat_row("Last Commit:    ", &last, last_color), c[3]);
    } else {
        frame.render_widget(
            Paragraph::new("No git repo detected").style(Style::default().fg(theme::SUBTEXT)).alignment(Alignment::Center),
            inner);
    }
}

fn render_repo_analysis(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Repo Analysis ")
        .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::OVERLAY))
        .style(Style::default().bg(theme::SURFACE));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(analysis) = &app.repo_analysis {
        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(Span::styled(
            format!("Commits: {}", analysis.total_commits),
            Style::default().fg(theme::SUBTEXT),
        )));
        lines.push(Line::from(vec![
            Span::styled("Insertions: ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(format!("+{}", analysis.total_insertions), Style::default().fg(theme::GREEN)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Deletions:  ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(format!("-{}", analysis.total_deletions), Style::default().fg(theme::RED)),
        ]));
        lines.push(Line::from(Span::styled(
            format!("Files: {}  Authors: {}", analysis.unique_files, analysis.unique_authors),
            Style::default().fg(theme::SUBTEXT),
        )));
        if let Some(first) = &analysis.first_commit {
            lines.push(Line::from(Span::styled(
                format!("First: {}", first.format("%Y-%m-%d")),
                Style::default().fg(theme::SUBTEXT),
            )));
        }
        if let Some(last) = &analysis.last_commit {
            lines.push(Line::from(Span::styled(
                format!("Last:  {}", last.format("%Y-%m-%d")),
                Style::default().fg(theme::SUBTEXT),
            )));
        }
        if !app.languages.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Languages",
                Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD),
            )));
            for lang in app.languages.iter().take(5) {
                lines.push(Line::from(Span::styled(
                    format!("  {} ({} files)", lang.name, lang.files),
                    Style::default().fg(theme::TEAL),
                )));
            }
        }
        frame.render_widget(Paragraph::new(lines), inner);
    } else {
        frame.render_widget(
            Paragraph::new("No repo detected")
                .style(Style::default().fg(theme::SUBTEXT))
                .alignment(Alignment::Center),
            inner,
        );
    }
}

fn render_skills_panel(frame: &mut Frame, area: Rect, app: &App) {
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
        .constraints(axes.iter().map(|_| Constraint::Length(3)).collect::<Vec<_>>())
        .spacing(1)
        .split(inner);

    for (i, axis) in axes.iter().enumerate().take(chunks.len()) {
        let sp = app.skill_progress.iter().find(|s| s.axis == *axis).cloned()
            .unwrap_or(devcore_devtrack::SkillProgress { axis: *axis, xp: 0, level: 0 });
        let ratio = (sp.xp as f64 / (MAX_LEVEL * XP_PER_LEVEL) as f64).min(1.0);
        let label = format!("{} (Lv {} - {} XP)", axis_name(axis), sp.level, sp.xp);
        let gauge = widgets::progress_gauge(&label, ratio, axis_gauge_color(axis))
            .block(Block::default().border_type(BorderType::Rounded)
                .borders(Borders::ALL).border_style(Style::default().fg(theme::OVERLAY)));
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
