use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::theme;
use devcore_challenges::{ProjectPack, ProjectProgress};

/// Renders the full-screen course viewer with header, stage navigator, content panel, and footer.
pub fn render_course_view(
    frame: &mut Frame,
    area: Rect,
    project: &ProjectPack,
    progress: Option<&ProjectProgress>,
    stage_cursor: usize,
    show_solutions: bool,
) {
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Min(0),    // body
            Constraint::Length(1), // footer
        ])
        .split(area);

    render_header(frame, outer_chunks[0], project, progress, stage_cursor);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(22), // stage navigator
            Constraint::Min(0),         // content (or content + solution split)
        ])
        .split(outer_chunks[1]);

    render_stage_navigator(
        frame,
        body_chunks[0],
        project,
        progress,
        stage_cursor,
    );

    if show_solutions {
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(body_chunks[1]);

        render_stage_content(frame, content_chunks[0], project, stage_cursor);
        render_solution_panel(frame, content_chunks[1], project, stage_cursor);
    } else {
        render_stage_content(frame, body_chunks[1], project, stage_cursor);
    }

    render_footer(frame, outer_chunks[2], show_solutions);
}

fn render_header(
    frame: &mut Frame,
    area: Rect,
    project: &ProjectPack,
    progress: Option<&ProjectProgress>,
    stage_cursor: usize,
) {
    let block = Block::default()
        .title(Span::styled(
            format!(" {} ", project.name),
            Style::default()
                .fg(theme::MAUVE)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::MAUVE))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12), // difficulty
            Constraint::Length(14), // language
            Constraint::Min(0),    // progress bar
            Constraint::Length(20), // stage indicator
        ])
        .split(inner);

    // Difficulty
    let diff_color = match project.difficulty.as_str() {
        "easy" => theme::GREEN,
        "medium" => theme::YELLOW,
        "hard" => theme::RED,
        _ => theme::SUBTEXT,
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("[{}]", project.difficulty),
            Style::default()
                .fg(diff_color)
                .add_modifier(Modifier::BOLD),
        ))),
        chunks[0],
    );

    // Language
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("{}", project.language),
            Style::default()
                .fg(theme::TEAL)
                .add_modifier(Modifier::BOLD),
        ))),
        chunks[1],
    );

    // Progress bar
    let total = project.stages.len();
    let completed = progress
        .map(|p| p.completed_stages.len())
        .unwrap_or(0);
    let ratio = if total > 0 {
        completed as f64 / total as f64
    } else {
        0.0
    };
    let pct = (ratio * 100.0) as u32;
    let progress_color = if progress.map_or(false, |p| p.is_complete()) {
        theme::GREEN
    } else {
        theme::YELLOW
    };
    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(progress_color)
                .bg(theme::BASE)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(ratio)
        .label(Span::styled(
            format!("{}/{} ({}%)", completed, total, pct),
            Style::default()
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(gauge, chunks[2]);

    // Stage indicator
    let stage_label = if !project.stages.is_empty() {
        format!(
            "Stage {}/{}",
            (stage_cursor + 1).min(total),
            total
        )
    } else {
        "No stages".to_string()
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            stage_label,
            Style::default()
                .fg(theme::BLUE)
                .add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Right),
        chunks[3],
    );
}

fn render_stage_navigator(
    frame: &mut Frame,
    area: Rect,
    project: &ProjectPack,
    progress: Option<&ProjectProgress>,
    stage_cursor: usize,
) {
    let block = Block::default()
        .title(Span::styled(
            " Stages ",
            Style::default()
                .fg(theme::TEAL)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::TEAL))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let completed_stages: Vec<usize> = progress
        .map(|p| p.completed_stages.clone())
        .unwrap_or_default();

    let items: Vec<ListItem> = project
        .stages
        .iter()
        .enumerate()
        .map(|(i, stage)| {
            let indicator = if completed_stages.contains(&i) {
                "[x]"
            } else if i == stage_cursor {
                "[>]"
            } else {
                "[ ]"
            };

            let indicator_color = if completed_stages.contains(&i) {
                theme::GREEN
            } else if i == stage_cursor {
                theme::YELLOW
            } else {
                theme::OVERLAY
            };

            let name_color = if i == stage_cursor {
                theme::TEXT
            } else {
                theme::SUBTEXT
            };

            let name_style = if i == stage_cursor {
                Style::default()
                    .fg(name_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(name_color)
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", indicator),
                    Style::default()
                        .fg(indicator_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{}. {}", i + 1, stage.name),
                    name_style,
                ),
            ]))
        })
        .collect();

    let list = List::new(items).highlight_style(
        Style::default()
            .bg(theme::OVERLAY)
            .add_modifier(Modifier::BOLD),
    );
    let mut state = ListState::default();
    state.select(Some(stage_cursor));
    frame.render_stateful_widget(list, inner, &mut state);
}

fn render_stage_content(
    frame: &mut Frame,
    area: Rect,
    project: &ProjectPack,
    stage_cursor: usize,
) {
    let stage = match project.stages.get(stage_cursor) {
        Some(s) => s,
        None => {
            frame.render_widget(
                Paragraph::new("No stage selected")
                    .style(Style::default().fg(theme::SUBTEXT))
                    .alignment(Alignment::Center),
                area,
            );
            return;
        }
    };

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Stage header: name + description
    let header_block = Block::default()
        .title(Span::styled(
            format!(" Stage {} - {} ", stage_cursor + 1, stage.name),
            Style::default()
                .fg(theme::BLUE)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BLUE))
        .style(Style::default().bg(theme::SURFACE));

    let header_inner = header_block.inner(outer[0]);
    frame.render_widget(header_block, outer[0]);
    frame.render_widget(
        Paragraph::new(stage.description.as_str())
            .style(Style::default().fg(theme::TEXT))
            .wrap(Wrap { trim: false }),
        header_inner,
    );

    // Content: skeleton files + test files
    let content_chunks = if stage.tests.is_empty() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(outer[1])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(outer[1])
    };

    // Skeleton code files
    let files_block = Block::default()
        .title(Span::styled(
            " Skeleton Code ",
            Style::default()
                .fg(theme::MAUVE)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::OVERLAY))
        .style(Style::default().bg(theme::BASE));

    let files_inner = files_block.inner(content_chunks[0]);
    frame.render_widget(files_block, content_chunks[0]);

    let mut code_lines: Vec<Line> = Vec::new();
    for file in &stage.files {
        // Filename header
        code_lines.push(Line::from(Span::styled(
            format!("── {} ", file.path),
            Style::default()
                .fg(theme::TEAL)
                .add_modifier(Modifier::BOLD),
        )));
        // Code with line numbers
        for (ln, line) in file.content.lines().enumerate() {
            code_lines.push(Line::from(vec![
                Span::styled(
                    format!("{:>4} │ ", ln + 1),
                    Style::default().fg(theme::OVERLAY),
                ),
                Span::styled(line.to_string(), Style::default().fg(theme::TEXT)),
            ]));
        }
        code_lines.push(Line::from(""));
    }
    frame.render_widget(
        Paragraph::new(code_lines)
            .wrap(Wrap { trim: false })
            .scroll((0, 0)),
        files_inner,
    );

    // Test files
    if !stage.tests.is_empty() && content_chunks.len() > 1 {
        let tests_block = Block::default()
            .title(Span::styled(
                " Tests ",
                Style::default()
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::OVERLAY))
            .style(Style::default().bg(theme::BASE));

        let tests_inner = tests_block.inner(content_chunks[1]);
        frame.render_widget(tests_block, content_chunks[1]);

        let mut test_lines: Vec<Line> = Vec::new();
        for file in &stage.tests {
            test_lines.push(Line::from(Span::styled(
                format!("── {} ", file.path),
                Style::default()
                    .fg(theme::TEAL)
                    .add_modifier(Modifier::BOLD),
            )));
            for (ln, line) in file.content.lines().enumerate() {
                test_lines.push(Line::from(vec![
                    Span::styled(
                        format!("{:>4} │ ", ln + 1),
                        Style::default().fg(theme::OVERLAY),
                    ),
                    Span::styled(line.to_string(), Style::default().fg(theme::TEXT)),
                ]));
            }
            test_lines.push(Line::from(""));
        }
        frame.render_widget(
            Paragraph::new(test_lines)
                .wrap(Wrap { trim: false })
                .scroll((0, 0)),
            tests_inner,
        );
    }
}

fn render_solution_panel(
    frame: &mut Frame,
    area: Rect,
    project: &ProjectPack,
    stage_cursor: usize,
) {
    let stage = match project.stages.get(stage_cursor) {
        Some(s) => s,
        None => return,
    };

    let block = Block::default()
        .title(Span::styled(
            " Solutions ",
            Style::default()
                .fg(theme::GREEN)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::GREEN))
        .style(Style::default().bg(theme::BASE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if stage.solution_files.is_empty() {
        frame.render_widget(
            Paragraph::new("No solutions available for this stage")
                .style(Style::default().fg(theme::SUBTEXT))
                .alignment(Alignment::Center),
            inner,
        );
        return;
    }

    let mut code_lines: Vec<Line> = Vec::new();
    for file in &stage.solution_files {
        code_lines.push(Line::from(Span::styled(
            format!("── {} ", file.path),
            Style::default()
                .fg(theme::TEAL)
                .add_modifier(Modifier::BOLD),
        )));
        for (ln, line) in file.content.lines().enumerate() {
            code_lines.push(Line::from(vec![
                Span::styled(
                    format!("{:>4} │ ", ln + 1),
                    Style::default().fg(theme::OVERLAY),
                ),
                Span::styled(line.to_string(), Style::default().fg(theme::GREEN)),
            ]));
        }
        code_lines.push(Line::from(""));
    }
    frame.render_widget(
        Paragraph::new(code_lines)
            .wrap(Wrap { trim: false })
            .scroll((0, 0)),
        inner,
    );
}

fn render_footer(frame: &mut Frame, area: Rect, show_solutions: bool) {
    let sol_label = if show_solutions {
        "[s] hide solution"
    } else {
        "[s] show solution"
    };
    let keybindings = vec![
        Span::styled(" Up/Down ", Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD)),
        Span::styled("navigate ", Style::default().fg(theme::SUBTEXT)),
        Span::styled("│", Style::default().fg(theme::OVERLAY)),
        Span::styled(
            format!(" {} ", sol_label),
            Style::default()
                .fg(theme::GREEN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("│", Style::default().fg(theme::OVERLAY)),
        Span::styled(" [c] ", Style::default().fg(theme::TEAL).add_modifier(Modifier::BOLD)),
        Span::styled("mark complete ", Style::default().fg(theme::SUBTEXT)),
        Span::styled("│", Style::default().fg(theme::OVERLAY)),
        Span::styled(" [Esc] ", Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)),
        Span::styled("back", Style::default().fg(theme::SUBTEXT)),
    ];
    frame.render_widget(
        Paragraph::new(Line::from(keybindings)).style(Style::default().bg(theme::SURFACE)),
        area,
    );
}
