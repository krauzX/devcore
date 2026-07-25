use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use crate::theme;
use crate::widgets;

pub fn render_dashboard(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .spacing(1)
        .split(area);

    render_semester_info(frame, chunks[0], app);
    render_sgpa_gauge(frame, chunks[1], app);
    render_stats_summary(frame, chunks[2], app);
    render_bottom(frame, chunks[3], app);
}

fn render_semester_info(frame: &mut Frame, area: Rect, app: &App) {
    let text = match &app.current_semester {
        Some(sem) => format!(
            "{} - {} ({}) | {}",
            app.config.institution, app.config.program, app.config.batch, sem.name
        ),
        None => format!(
            "{} - {} ({}) | No semester set",
            app.config.institution, app.config.program, app.config.batch
        ),
    };
    frame.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(theme::TEAL))
            .alignment(Alignment::Center),
        area,
    );
}

fn render_sgpa_gauge(frame: &mut Frame, area: Rect, app: &App) {
    let ratio = app.sgpa.map(|s| s / 10.0).unwrap_or(0.0);
    let label = match app.sgpa {
        Some(s) => format!("SGPA: {:.2}", s),
        None => "SGPA: --".to_string(),
    };
    let gauge = widgets::progress_gauge(&label, ratio, theme::GREEN).block(
        Block::default()
            .title(" Current SGPA ")
            .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY))
            .style(Style::default().bg(theme::SURFACE)),
    );
    frame.render_widget(gauge, area);
}

fn render_stats_summary(frame: &mut Frame, area: Rect, app: &App) {
    let streak = app.streak.as_ref().map(|s| {
        format!("{}d / {}d / {}d", s.current, s.longest, s.total_days)
    }).unwrap_or_else(|| "No git repo".to_string());

    let line = Line::from(vec![
        Span::styled(format!("Courses: {} ", app.course_count), Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled("│", Style::default().fg(theme::OVERLAY)),
        Span::styled(format!(" Credits: {} ", app.total_credits), Style::default().fg(theme::TEAL).add_modifier(Modifier::BOLD)),
        Span::styled("│", Style::default().fg(theme::OVERLAY)),
        Span::styled(format!(" Deadlines: {} ", app.upcoming_deadlines.len()), Style::default().fg(theme::PEACH).add_modifier(Modifier::BOLD)),
        Span::styled("│", Style::default().fg(theme::OVERLAY)),
        Span::styled(format!(" {} ", streak), Style::default().fg(theme::GREEN)),
    ]);

    frame.render_widget(
        Paragraph::new(line).style(Style::default().bg(theme::SURFACE)).alignment(Alignment::Center),
        area,
    );
}

fn render_bottom(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .spacing(1)
        .split(area);

    render_deadline_list(frame, chunks[0], app);
    render_quick_actions(frame, chunks[1], app);
}

fn render_deadline_list(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(format!(" Upcoming Deadlines ({}) ", app.upcoming_deadlines.len()))
        .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::OVERLAY))
        .style(Style::default().bg(theme::SURFACE));
    if app.upcoming_deadlines.is_empty() {
        frame.render_widget(
            Paragraph::new("No upcoming deadlines")
                .style(Style::default().fg(theme::SUBTEXT))
                .alignment(Alignment::Center).block(block), area);
        return;
    }
    let today = chrono::Local::now().naive_local().date();
    let items: Vec<ListItem> = app.upcoming_deadlines.iter().take(8).map(|d| {
        let days = (d.due_date - today).num_days();
        widgets::deadline_item(&d.title, days)
    }).collect();
    frame.render_widget(List::new(items).block(block), area);
}

fn render_quick_actions(frame: &mut Frame, area: Rect, app: &App) {
    let mut items = vec![
        ListItem::new(Line::from(Span::styled("[1] Dashboard", Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)))),
        ListItem::new(Line::from(Span::styled("[2] Academic", Style::default().fg(theme::TEXT)))),
        ListItem::new(Line::from(Span::styled("[3] Git", Style::default().fg(theme::TEXT)))),
        ListItem::new(Line::from(Span::styled("[4] Challenges", Style::default().fg(theme::TEXT)))),
        ListItem::new(Line::from("")),
    ];

    if let Some(cgpa) = app.cgpa {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("  CGPA: ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(format!("{:.2}", cgpa), Style::default().fg(theme::GREEN).add_modifier(Modifier::BOLD)),
        ])));
    }

    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(Span::styled("[TAB] Next  |  [q] Quit", Style::default().fg(theme::OVERLAY)))));

    let block = Block::default()
        .title(" Quick Actions ")
        .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::OVERLAY))
        .style(Style::default().bg(theme::SURFACE));

    frame.render_widget(List::new(items).block(block), area);
}
