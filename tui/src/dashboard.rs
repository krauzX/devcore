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
            Constraint::Min(0),
        ])
        .spacing(1)
        .split(area);

    let semester_text = match &app.current_semester {
        Some(sem) => format!(
            "{} - {} ({}) | {} | SGPA: {}",
            app.config.institution, app.config.program, app.config.batch, sem.name, format_sgpa(app.sgpa)
        ),
        None => format!(
            "{} - {} ({}) | No semester | SGPA: {}",
            app.config.institution, app.config.program, app.config.batch, format_sgpa(app.sgpa)
        ),
    };
    let semester_para = Paragraph::new(semester_text)
        .style(Style::default().fg(theme::TEAL))
        .alignment(Alignment::Center);
    frame.render_widget(semester_para, chunks[0]);

    let streak_text = match &app.streak {
        Some(s) => format!(
            "Streak: {}d current / {}d longest / {}d total",
            s.current, s.longest, s.total_days
        ),
        None => "No git repo detected".to_string(),
    };
    let streak_para = Paragraph::new(streak_text)
        .style(Style::default().fg(theme::GREEN))
        .alignment(Alignment::Center);
    frame.render_widget(streak_para, chunks[1]);

    let chunks2 = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .spacing(1)
        .split(chunks[2]);

    let overview_items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("Deadlines (30d): ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(
                app.upcoming_deadlines.len().to_string(),
                Style::default().fg(theme::YELLOW),
            ),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("Packs available: ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(
                app.packs.len().to_string(),
                Style::default().fg(theme::YELLOW),
            ),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("Packs installed: ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(
                app.installed_count.to_string(),
                Style::default().fg(theme::GREEN),
            ),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("Problems solved: ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(
                app.solved_count.to_string(),
                Style::default().fg(theme::GREEN),
            ),
        ])),
    ];

    let overview = List::new(overview_items)
        .block(
            Block::default()
                .title(" Overview ")
                .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::OVERLAY))
                .style(Style::default().bg(theme::SURFACE)),
        );
    frame.render_widget(overview, chunks2[0]);

    let actions = vec![
        ListItem::new(Line::from(Span::styled(
            "[1] Dashboard",
            Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD),
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
        ListItem::new(Line::from(Span::styled(
            "[TAB] Next  |  [q] Quit",
            Style::default().fg(theme::OVERLAY),
        ))),
    ];

    let actions_list = List::new(actions)
        .block(
            Block::default()
                .title(" Quick Actions ")
                .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::OVERLAY))
                .style(Style::default().bg(theme::SURFACE)),
        );
    frame.render_widget(actions_list, chunks2[1]);
}

fn format_sgpa(sgpa: Option<f64>) -> String {
    match sgpa {
        Some(v) => format!("{:.2}", v),
        None => "--".to_string(),
    }
}
