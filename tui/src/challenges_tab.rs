use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;
use crate::theme;
use devcore_challenges::Difficulty;

pub fn render_challenges_tab(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Percentage(35),
            Constraint::Min(10),
        ])
        .spacing(1)
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .spacing(1)
        .split(chunks[0]);

    render_packs_panel(frame, top_chunks[0], app);
    render_problems_panel(frame, top_chunks[1], app);
    render_offline_problems(frame, chunks[2], app);
}

fn difficulty_color(diff: &Difficulty) -> Color {
    match diff {
        Difficulty::Easy => theme::GREEN,
        Difficulty::Medium => theme::YELLOW,
        Difficulty::Hard => theme::RED,
    }
}

fn render_packs_panel(frame: &mut Frame, area: Rect, app: &App) {
    let header = Row::new(vec![
        Cell::from(Span::styled("Difficulty", Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Pack Name", Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Problems", Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD))),
    ]).style(Style::default().bg(theme::BASE));

    let rows: Vec<Row> = app.packs.iter().map(|pack| {
        let color = difficulty_color(&pack.difficulty);
        Row::new(vec![
            Cell::from(Span::styled(format!("[{}]", pack.difficulty), Style::default().fg(color))),
            Cell::from(Span::styled(pack.name.clone(), Style::default().fg(theme::TEXT))),
            Cell::from(Span::styled(format!("{}", pack.problems.len()), Style::default().fg(theme::SUBTEXT))),
        ])
    }).collect();

    frame.render_widget(Table::new(rows, [
        Constraint::Length(10), Constraint::Min(15), Constraint::Length(10),
    ]).header(header).block(
        Block::default()
            .title(format!(" Available Packs ({}) ", app.packs.len()))
            .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY))
            .style(Style::default().bg(theme::SURFACE)),
    ).column_spacing(1), area);
}

fn render_problems_panel(frame: &mut Frame, area: Rect, app: &App) {
    let header = Row::new(vec![
        Cell::from(Span::styled("Difficulty", Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Problem Name", Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Hints", Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD))),
    ]).style(Style::default().bg(theme::BASE));
    let rows: Vec<Row> = app.packs.iter().flat_map(|pack| {
        pack.problems.iter().map(move |problem| {
            let color = difficulty_color(&problem.difficulty);
            Row::new(vec![
                Cell::from(Span::styled(format!("[{}]", problem.difficulty), Style::default().fg(color))),
                Cell::from(Span::styled(problem.name.clone(), Style::default().fg(theme::TEXT))),
                Cell::from(Span::styled(format!("{}", problem.hints.len()), Style::default().fg(theme::SUBTEXT))),
            ])
        })
    }).collect();
    frame.render_widget(Table::new(rows, [
        Constraint::Length(10), Constraint::Min(20), Constraint::Length(8),
    ]).header(header).block(
        Block::default().title(" All Problems ")
            .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
            .border_type(BorderType::Rounded).borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY))
            .style(Style::default().bg(theme::SURFACE)),
    ).column_spacing(1), area);
}

fn render_offline_problems(frame: &mut Frame, area: Rect, app: &App) {
    let header = Row::new(vec![
        Cell::from(Span::styled("ID", Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Title", Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Difficulty", Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Accept%", Style::default().fg(theme::MAUVE).add_modifier(Modifier::BOLD))),
    ]).style(Style::default().bg(theme::BASE));

    let rows: Vec<Row> = app.offline_problems.iter().map(|p| {
        let color = difficulty_color(&p.difficulty);
        Row::new(vec![
            Cell::from(Span::styled(format!("{}", p.fid), Style::default().fg(theme::SUBTEXT))),
            Cell::from(Span::styled(p.title.clone(), Style::default().fg(theme::TEXT))),
            Cell::from(Span::styled(format!("[{}]", p.difficulty), Style::default().fg(color))),
            Cell::from(Span::styled(format!("{:.1}%", p.acceptance), Style::default().fg(theme::SUBTEXT))),
        ])
    }).collect();

    let title = format!(
        " Offline Problems — Page {}/{} ({} total) ",
        app.offline_page, app.offline_total_pages, app.offline_total
    );

    frame.render_widget(Table::new(rows, [
        Constraint::Length(6), Constraint::Min(20), Constraint::Length(10), Constraint::Length(10),
    ]).header(header).block(
        Block::default()
            .title(title)
            .title_style(Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY))
            .style(Style::default().bg(theme::SURFACE)),
    ).column_spacing(1), area);
}
