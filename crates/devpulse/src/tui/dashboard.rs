use crate::tui::app::App;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_stats(f, chunks[0], app);
    render_activity(f, chunks[1], app);
}

fn render_stats(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(0)])
        .split(area);

    let session_text = if let Some(start) = app.session_start {
        let elapsed = start.elapsed();
        let mins = elapsed.as_secs() / 60;
        let secs = elapsed.as_secs() % 60;
        vec![
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "ACTIVE",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Timer:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:02}:{:02}", mins, secs),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Press 's' to stop session",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
                Span::styled("INACTIVE", Style::default().fg(Color::Red)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Press 's' to start tracking",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    };

    let session_block = Paragraph::new(session_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Session ")
            .border_style(Style::default().fg(Color::Green)),
    );
    f.render_widget(session_block, chunks[0]);

    let today_mins = app.total_minutes;
    let today_hours = today_mins / 60.0;
    let commit_count = app.commits_today.len();
    let event_count = app.events_today.len();
    let ai_count = app
        .commits_today
        .iter()
        .filter(|c| c.is_ai_generated)
        .count();
    let human_count = commit_count - ai_count;

    let stats = vec![
        Line::from(vec![
            Span::styled("Today:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.1}h ({:.0}m)", today_hours, today_mins),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("Commits:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{} total", commit_count),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("  AI:       ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", ai_count), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  Human:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", human_count),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled("Events:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", event_count),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let stats_block = Paragraph::new(stats).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Stats (24h) ")
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(stats_block, chunks[1]);
}

fn render_activity(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(8)])
        .split(area);

    let mut rows = Vec::new();
    for hour in 0..24 {
        let mins = app.hourly_activity.get(&hour).copied().unwrap_or(0);
        let bar_len = if mins > 0 {
            let max = app.hourly_activity.values().copied().max().unwrap_or(1);
            ((mins as f64 / max as f64) * 25.0) as usize
        } else {
            0
        };

        let color = if mins == 0 {
            Color::DarkGray
        } else if mins > 30 {
            Color::Red
        } else if mins > 15 {
            Color::Yellow
        } else {
            Color::Green
        };

        let bar = "█".repeat(bar_len);
        rows.push(Row::new(vec![
            Cell::from(format!("{:>2}", hour)).style(Style::default().fg(Color::DarkGray)),
            Cell::from(format!("{:>3}m", mins)),
            Cell::from(bar).style(Style::default().fg(color)),
        ]));
    }

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(0),
        ],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Hourly Activity ")
            .border_style(Style::default().fg(Color::Cyan)),
    )
    .header(
        Row::new(vec!["Hr", "Min", "Activity"]).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    );

    f.render_widget(table, chunks[0]);

    let categories: Vec<ListItem> = app
        .categories
        .iter()
        .map(|(cat, mins)| {
            let pct = if app.total_minutes > 0.0 {
                (mins / app.total_minutes) * 100.0
            } else {
                0.0
            };
            let bar_len = (pct / 100.0 * 20.0) as usize;
            let color = if pct > 40.0 {
                Color::Red
            } else if pct > 20.0 {
                Color::Yellow
            } else {
                Color::Green
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<14}", cat), Style::default().fg(Color::White)),
                Span::styled(format!(" {:>5.1}% ", pct), Style::default().fg(color)),
                Span::styled("█".repeat(bar_len), Style::default().fg(color)),
            ]))
        })
        .collect();

    let cat_list = List::new(categories).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Time Breakdown ")
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(cat_list, chunks[1]);
}
