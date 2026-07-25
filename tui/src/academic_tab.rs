use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use crate::theme;

pub fn render_academic_tab(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
        ])
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

    // SGPA gauge
    let sgpa_ratio = app.sgpa.map(|s| s / 10.0).unwrap_or(0.0);
    let sgpa_label = app.sgpa.map(|s| format!("{:.2}", s)).unwrap_or_else(|| "--".to_string());
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(" SGPA ")
                .title_style(Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD))
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
            Style::default().fg(theme::TEXT).add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(gauge, chunks[0]);

    // Course count
    let course_info = Paragraph::new(Line::from(vec![
        Span::styled("Courses: ", Style::default().fg(theme::SUBTEXT)),
        Span::styled(
            app.course_count.to_string(),
            Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY))
            .style(Style::default().bg(theme::SURFACE)),
    );
    frame.render_widget(course_info, chunks[1]);

    // Total credits
    let credits_info = Paragraph::new(Line::from(vec![
        Span::styled("Credits: ", Style::default().fg(theme::SUBTEXT)),
        Span::styled(
            app.total_credits.to_string(),
            Style::default().fg(theme::TEAL).add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY))
            .style(Style::default().bg(theme::SURFACE)),
    );
    frame.render_widget(credits_info, chunks[2]);
}

fn render_content(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .spacing(1)
        .split(area);

    render_semester_list(frame, chunks[0], app);
    render_deadlines(frame, chunks[1], app);
}

fn render_semester_list(frame: &mut Frame, area: Rect, app: &App) {
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

    let list = List::new(items).block(
        Block::default()
            .title(" Semesters ")
            .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY))
            .style(Style::default().bg(theme::SURFACE)),
    );
    frame.render_widget(list, area);
}

fn deadline_urgency_color(days_left: i64) -> Color {
    if days_left <= 1 {
        theme::RED
    } else if days_left <= 7 {
        theme::YELLOW
    } else {
        theme::GREEN
    }
}

fn deadline_priority_str(days_left: i64) -> &'static str {
    if days_left <= 0 {
        "OVERDUE"
    } else if days_left <= 1 {
        "!!!"
    } else if days_left <= 3 {
        "!!"
    } else if days_left <= 7 {
        "!"
    } else {
        ""
    }
}

fn render_deadlines(frame: &mut Frame, area: Rect, app: &App) {
    if app.upcoming_deadlines.is_empty() {
        let empty = Paragraph::new("No upcoming deadlines in the next 30 days")
            .style(Style::default().fg(theme::OVERLAY))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Upcoming Deadlines (30d) ")
                    .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::OVERLAY))
                    .style(Style::default().bg(theme::SURFACE)),
            );
        frame.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from(Span::styled(
            "Urgency",
            Style::default()
                .fg(theme::MAUVE)
                .add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Title",
            Style::default()
                .fg(theme::MAUVE)
                .add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Due Date",
            Style::default()
                .fg(theme::MAUVE)
                .add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Days",
            Style::default()
                .fg(theme::MAUVE)
                .add_modifier(Modifier::BOLD),
        )),
    ])
    .style(Style::default().bg(theme::BASE));

    let today = chrono::Local::now().naive_local().date();

    let rows: Vec<Row> = app
        .upcoming_deadlines
        .iter()
        .map(|d| {
            let days_left = (d.due_date - today).num_days();
            let color = deadline_urgency_color(days_left);
            let priority = deadline_priority_str(days_left);

            Row::new(vec![
                Cell::from(Span::styled(
                    format!("{:<8}", priority),
                    Style::default()
                        .fg(color)
                        .add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    d.title.clone(),
                    Style::default().fg(theme::TEXT),
                )),
                Cell::from(Span::styled(
                    d.due_date.to_string(),
                    Style::default().fg(color),
                )),
                Cell::from(Span::styled(
                    days_left.to_string(),
                    Style::default().fg(color),
                )),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Min(20),
            Constraint::Length(12),
            Constraint::Length(5),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(" Upcoming Deadlines ({}) ", app.upcoming_deadlines.len()))
            .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY))
            .style(Style::default().bg(theme::SURFACE)),
    )
    .column_spacing(1);
    frame.render_widget(table, area);
}
