use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use crate::theme;

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
    render_stats_bar(frame, chunks[2], app);
    render_bottom(frame, chunks[3], app);
}

fn render_semester_info(frame: &mut Frame, area: Rect, app: &App) {
    let semester_text = match &app.current_semester {
        Some(sem) => format!(
            "{} - {} ({}) | {}",
            app.config.institution, app.config.program, app.config.batch, sem.name
        ),
        None => format!(
            "{} - {} ({}) | No semester set",
            app.config.institution, app.config.program, app.config.batch
        ),
    };
    let semester_para = Paragraph::new(semester_text)
        .style(Style::default().fg(theme::TEAL))
        .alignment(Alignment::Center);
    frame.render_widget(semester_para, area);
}

fn render_sgpa_gauge(frame: &mut Frame, area: Rect, app: &App) {
    let sgpa_ratio = app.sgpa.map(|s| s / 10.0).unwrap_or(0.0);
    let sgpa_label = match app.sgpa {
        Some(s) => format!("SGPA: {:.2}", s),
        None => "SGPA: --".to_string(),
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" Current SGPA ")
                .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::OVERLAY))
                .style(Style::default().bg(theme::SURFACE)),
        )
        .gauge_style(
            Style::default()
                .fg(theme::GREEN)
                .bg(theme::BASE)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(sgpa_ratio)
        .label(Span::styled(
            sgpa_label,
            Style::default()
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(gauge, area);
}

fn render_stats_bar(frame: &mut Frame, area: Rect, app: &App) {
    let streak_text = match &app.streak {
        Some(s) => format!(
            "Streak: {}d current / {}d longest / {}d total",
            s.current, s.longest, s.total_days
        ),
        None => "No git repo detected".to_string(),
    };

    let stats = Line::from(vec![
        Span::styled(
            format!("Courses: {} ", app.course_count),
            Style::default()
                .fg(theme::YELLOW)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("│", Style::default().fg(theme::OVERLAY)),
        Span::styled(
            format!(" Credits: {} ", app.total_credits),
            Style::default()
                .fg(theme::TEAL)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("│", Style::default().fg(theme::OVERLAY)),
        Span::styled(
            format!(" Deadlines: {} ", app.upcoming_deadlines.len()),
            Style::default()
                .fg(theme::PEACH)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("│", Style::default().fg(theme::OVERLAY)),
        Span::styled(
            format!(" {} ", streak_text),
            Style::default().fg(theme::GREEN),
        ),
    ]);

    let para = Paragraph::new(stats)
        .style(Style::default().bg(theme::SURFACE))
        .alignment(Alignment::Center);
    frame.render_widget(para, area);
}

fn render_bottom(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .spacing(1)
        .split(area);

    render_deadlines_list(frame, chunks[0], app);
    render_quick_actions(frame, chunks[1], app);
}

fn deadline_color(days_left: i64) -> Color {
    if days_left <= 1 {
        theme::RED
    } else if days_left <= 3 {
        theme::YELLOW
    } else {
        theme::GREEN
    }
}

fn render_deadlines_list(frame: &mut Frame, area: Rect, app: &App) {
    let today = chrono::Local::now().naive_local().date();

    if app.upcoming_deadlines.is_empty() {
        let empty = Paragraph::new("No upcoming deadlines")
            .style(Style::default().fg(theme::OVERLAY))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Upcoming Deadlines ")
                    .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::OVERLAY))
                    .style(Style::default().bg(theme::SURFACE)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .upcoming_deadlines
        .iter()
        .take(8)
        .map(|d| {
            let days_left = (d.due_date - today).num_days();
            let color = deadline_color(days_left);
            let priority = if days_left <= 0 {
                "OVERDUE"
            } else if days_left <= 1 {
                "!!!"
            } else if days_left <= 3 {
                "!!"
            } else if days_left <= 7 {
                "!"
            } else {
                ""
            };

            let prefix = if priority.is_empty() {
                format!("     ")
            } else {
                format!("[{:<3}]", priority)
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    prefix,
                    Style::default()
                        .fg(color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    d.title.clone(),
                    Style::default().fg(theme::TEXT),
                ),
                Span::styled(
                    format!(" ({}) ", days_left),
                    Style::default().fg(color),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(
                " Upcoming Deadlines ({}) ",
                app.upcoming_deadlines.len()
            ))
            .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY))
            .style(Style::default().bg(theme::SURFACE)),
    );
    frame.render_widget(list, area);
}

fn render_quick_actions(frame: &mut Frame, area: Rect, app: &App) {
    let mut items = vec![
        ListItem::new(Line::from(Span::styled(
            "[1] Dashboard",
            Style::default()
                .fg(theme::YELLOW)
                .add_modifier(Modifier::BOLD),
        ))),
        ListItem::new(Line::from(Span::styled(
            "[2] Academic",
            Style::default().fg(theme::TEXT),
        ))),
        ListItem::new(Line::from(Span::styled(
            "[3] Git",
            Style::default().fg(theme::TEXT),
        ))),
        ListItem::new(Line::from(Span::styled(
            "[4] Challenges",
            Style::default().fg(theme::TEXT),
        ))),
        ListItem::new(Line::from("")),
    ];

    if let Some(cgpa) = app.cgpa {
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                "  CGPA: ",
                Style::default().fg(theme::SUBTEXT),
            ),
            Span::styled(
                format!("{:.2}", cgpa),
                Style::default()
                    .fg(theme::GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
        ])));
    }

    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(Span::styled(
        "[TAB] Next  |  [q] Quit",
        Style::default().fg(theme::OVERLAY),
    ))));

    let actions_list = List::new(items).block(
        Block::default()
            .title(" Quick Actions ")
            .title_style(
                Style::default()
                    .fg(theme::BLUE)
                    .add_modifier(Modifier::BOLD),
            )
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY))
            .style(Style::default().bg(theme::SURFACE)),
    );
    frame.render_widget(actions_list, area);
}
