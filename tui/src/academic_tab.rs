use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use crate::theme;
use crate::widgets;

pub fn render_academic_tab(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .spacing(1)
        .split(area);

    render_stats_bar(frame, chunks[0], app);
    render_content(frame, chunks[1], app);
}

fn render_stats_bar(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .spacing(1)
        .split(area);

    let ratio = app.sgpa.map(|s| s / 10.0).unwrap_or(0.0);
    let label = app
        .sgpa
        .map(|s| format!("{:.2}", s))
        .unwrap_or_else(|| "--".to_string());
    let gauge = widgets::progress_gauge(&label, ratio, theme::GREEN).block(
        Block::default()
            .title(" SGPA ")
            .title_style(
                Style::default()
                    .fg(theme::MAUVE)
                    .add_modifier(Modifier::BOLD),
            )
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY))
            .style(Style::default().bg(theme::SURFACE)),
    );
    frame.render_widget(gauge, chunks[0]);

    frame.render_widget(
        widgets::stat_row("Courses: ", &app.course_count.to_string(), theme::YELLOW)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::OVERLAY))
                    .style(Style::default().bg(theme::SURFACE)),
            ),
        chunks[1],
    );

    frame.render_widget(
        widgets::stat_row(
            "Credits: ",
            &app.total_credits.to_string(),
            theme::TEAL,
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::OVERLAY))
                .style(Style::default().bg(theme::SURFACE)),
        ),
        chunks[2],
    );
}

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .spacing(1)
        .split(area);

    render_semester_panel(frame, chunks[0], app);
    render_deadline_panel(frame, chunks[1], app);
}

fn render_semester_panel(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Semesters ")
        .title_style(
            Style::default()
                .fg(theme::BLUE)
                .add_modifier(Modifier::BOLD),
        )
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::OVERLAY))
        .style(Style::default().bg(theme::SURFACE));

    if app.semesters.is_empty() {
        frame.render_widget(
            Paragraph::new("No semesters yet. Press 's' to select one.")
                .style(Style::default().fg(theme::OVERLAY))
                .alignment(Alignment::Center)
                .block(block),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = app
        .semesters
        .iter()
        .map(|sem| {
            let marker = if sem.is_current { " > " } else { "   " };
            let style = if sem.is_current {
                Style::default()
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT)
            };
            ListItem::new(Line::from(Span::styled(
                format!("{}{}", marker, sem.name),
                style,
            )))
        })
        .collect();

    frame.render_widget(List::new(items).block(block), area);
}

fn render_deadline_panel(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(format!(
            " Upcoming Deadlines ({}) — {}d window ",
            app.upcoming_deadlines.len(),
            app.deadline_days
        ))
        .title_style(
            Style::default()
                .fg(theme::BLUE)
                .add_modifier(Modifier::BOLD),
        )
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::OVERLAY))
        .style(Style::default().bg(theme::SURFACE));

    if app.upcoming_deadlines.is_empty() {
        frame.render_widget(
            Paragraph::new(format!("No upcoming deadlines in the next {} days", app.deadline_days))
                .style(Style::default().fg(theme::OVERLAY))
                .alignment(Alignment::Center)
                .block(block),
            area,
        );
        return;
    }

    let today = chrono::Local::now().naive_local().date();
    let items: Vec<ListItem> = app
        .upcoming_deadlines
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let days = (d.due_date - today).num_days();
            let selected = app.selected_deadline_index == Some(i);
            if selected {
                let urgency = devcore_academic::UrgencyLevel::from_days_left(days);
                let color = widgets::urgency_color(urgency);
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("▸ {} ({}d) ", d.title, days),
                        Style::default()
                            .fg(color)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                    ),
                ]))
            } else {
                widgets::deadline_item(&d.title, days)
            }
        })
        .collect();

    frame.render_widget(List::new(items).block(block), area);
}
