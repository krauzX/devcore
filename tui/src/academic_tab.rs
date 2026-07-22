use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use crate::theme;

pub fn render_academic_tab(frame: &mut Frame, area: Rect, app: &App) {
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
            "Due Date",
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
            "Status",
            Style::default()
                .fg(theme::MAUVE)
                .add_modifier(Modifier::BOLD),
        )),
    ])
    .style(Style::default().bg(theme::BASE));

    let rows: Vec<Row> = app
        .upcoming_deadlines
        .iter()
        .map(|d| {
            let status_style = if d.completed {
                Style::default().fg(theme::GREEN)
            } else {
                Style::default().fg(theme::PEACH)
            };
            Row::new(vec![
                Cell::from(Span::styled(
                    d.due_date.to_string(),
                    Style::default().fg(theme::YELLOW),
                )),
                Cell::from(Span::styled(
                    d.title.clone(),
                    Style::default().fg(theme::TEXT),
                )),
                Cell::from(Span::styled(
                    if d.completed { "  [x]  " } else { "  [ ]  " },
                    status_style,
                )),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Min(20),
            Constraint::Length(8),
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
