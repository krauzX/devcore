use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render_academic_tab(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
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
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
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
            .borders(Borders::ALL),
    );
    frame.render_widget(list, area);
}

fn render_deadlines(frame: &mut Frame, area: Rect, app: &App) {
    if app.upcoming_deadlines.is_empty() {
        let empty = Paragraph::new("No upcoming deadlines in the next 30 days")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" Upcoming Deadlines (30d) ")
                    .borders(Borders::ALL),
            );
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .upcoming_deadlines
        .iter()
        .map(|d| {
            let status = if d.completed { " [x] " } else { " [ ] " };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", d.due_date),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    d.title.clone(),
                    Style::default().fg(Color::White),
                ),
                Span::styled(status, Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(" Upcoming Deadlines ({}) ", app.upcoming_deadlines.len()))
            .borders(Borders::ALL),
    );
    frame.render_widget(list, area);
}
