use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use devcore_challenges::Difficulty;

pub fn render_challenges_tab(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_packs(frame, chunks[0], app);
    render_problems(frame, chunks[1], app);
}

fn difficulty_color(diff: &Difficulty) -> Color {
    match diff {
        Difficulty::Easy => Color::Green,
        Difficulty::Medium => Color::Yellow,
        Difficulty::Hard => Color::Red,
    }
}

fn render_packs(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .packs
        .iter()
        .map(|pack| {
            let color = difficulty_color(&pack.difficulty);
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("[{}] ", pack.difficulty),
                    Style::default().fg(color),
                ),
                Span::styled(
                    pack.name.clone(),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!(" ({} problems)", pack.problems.len()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(" Available Packs ({}) ", app.packs.len()))
            .borders(Borders::ALL),
    );
    frame.render_widget(list, area);
}

fn render_problems(frame: &mut Frame, area: Rect, app: &App) {
    let all_problems: Vec<_> = app
        .packs
        .iter()
        .flat_map(|pack| {
            pack.problems.iter().map(move |problem| {
                let color = difficulty_color(&problem.difficulty);
                let hint_count = problem.hints.len();
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("[{}] ", problem.difficulty),
                        Style::default().fg(color),
                    ),
                    Span::styled(
                        problem.name.clone(),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!(" ({} hints)", hint_count),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
        })
        .collect();

    let list = List::new(all_problems).block(
        Block::default()
            .title(" All Problems ")
            .borders(Borders::ALL),
    );
    frame.render_widget(list, area);
}
