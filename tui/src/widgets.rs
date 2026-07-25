use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::XpField;
use crate::theme;
use devcore_academic::UrgencyLevel;
use devcore_challenges::{Problem, ProjectPack};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Success,
    Error,
    Info,
}

pub struct KeyBinding {
    pub key: &'static str,
    pub label: &'static str,
    pub color: Color,
}

pub fn status_bar(frame: &mut Frame, area: Rect, keybindings: &[KeyBinding], status_msg: Option<&str>) {
    let mut spans: Vec<Span> = Vec::new();
    if let Some(msg) = status_msg {
        let msg_color = if msg.starts_with("Failed") || msg.starts_with("Error") || msg.starts_with("Invalid") || msg.starts_with("No ") {
            theme::RED
        } else {
            theme::GREEN
        };
        spans.push(Span::styled(
            format!(" {} ", msg),
            Style::default().fg(msg_color).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("  ", Style::default()));
    }
    for (i, kb) in keybindings.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("│", Style::default().fg(theme::OVERLAY)));
        }
        spans.push(Span::styled(
            format!(" {} ", kb.key),
            Style::default().fg(kb.color).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!("{} ", kb.label),
            Style::default().fg(theme::SUBTEXT),
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(theme::SURFACE)),
        area,
    );
}

pub fn section_header(title: &str, color: Color) -> Paragraph<'_> {
    Paragraph::new(Line::from(Span::styled(
        title,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )))
}

pub fn progress_gauge(label: &str, ratio: f64, color: Color) -> Gauge<'_> {
    Gauge::default()
        .gauge_style(
            Style::default().fg(color).bg(theme::BASE).add_modifier(Modifier::BOLD),
        )
        .ratio(ratio)
        .label(Span::styled(
            label,
            Style::default().fg(theme::TEXT).add_modifier(Modifier::BOLD),
        ))
}

pub fn stat_row<'a>(label: &'a str, value: &'a str, color: Color) -> Paragraph<'a> {
    Paragraph::new(Line::from(vec![
        Span::styled(label, Style::default().fg(theme::SUBTEXT)),
        Span::styled(
            value,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ]))
}

pub fn deadline_item(title: &str, days_left: i64) -> ListItem<'_> {
    let urgency = UrgencyLevel::from_days_left(days_left);
    let color = urgency_color(urgency);
    let prefix = urgency_label_padded(urgency);
    ListItem::new(Line::from(vec![
        Span::styled(
            prefix,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(title.to_string(), Style::default().fg(theme::TEXT)),
        Span::styled(
            format!(" ({}) ", days_left),
            Style::default().fg(color),
        ),
    ]))
}

pub fn urgency_color(level: UrgencyLevel) -> Color {
    match level {
        UrgencyLevel::Overdue | UrgencyLevel::Critical => theme::RED,
        UrgencyLevel::Warning => theme::YELLOW,
        UrgencyLevel::Soon | UrgencyLevel::Normal => theme::GREEN,
    }
}

fn urgency_label_padded(level: UrgencyLevel) -> String {
    match level {
        UrgencyLevel::Overdue => "[OVR]".to_string(),
        UrgencyLevel::Critical => "[!!!]".to_string(),
        UrgencyLevel::Warning => " [!!]".to_string(),
        UrgencyLevel::Soon => "  [!]".to_string(),
        UrgencyLevel::Normal => "     ".to_string(),
    }
}

pub fn render_input_form(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    labels: &[&str],
    values: &[String],
    current_field: usize,
    status_msg: Option<&str>,
    status_kind: StatusKind,
) {
    let mut lines: Vec<Line> = Vec::new();

    for (i, (label, value)) in labels.iter().zip(values.iter()).enumerate() {
        let is_active = i == current_field;
        let marker = if is_active { "> " } else { "  " };
        let label_style = if is_active {
            Style::default()
                .fg(theme::YELLOW)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::SUBTEXT)
        };

        let value_span = if is_active {
            Span::styled(
                format!("{}|", value),
                Style::default().fg(theme::TEXT),
            )
        } else {
            Span::styled(value.as_str(), Style::default().fg(theme::TEXT))
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  {} {}: ", marker, label), label_style),
            value_span,
        ]));
    }

    lines.push(Line::from(""));

    if let Some(msg) = status_msg {
        let color = match status_kind {
            StatusKind::Success => theme::GREEN,
            StatusKind::Error => theme::RED,
            StatusKind::Info => theme::YELLOW,
        };
        lines.push(Line::from(Span::styled(
            format!("  {}", msg),
            Style::default().fg(color),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "  Tab: field │ Enter: submit │ Esc: cancel",
            Style::default().fg(theme::SUBTEXT),
        )));
    }

    let form_height = lines.len() as u16 + 2;
    let form_width = 50.min(area.width.saturating_sub(4));

    if form_height > area.height || form_width < 20 {
        return;
    }

    let x = area.x + (area.width.saturating_sub(form_width)) / 2;
    let y = area.y + area.height.saturating_sub(form_height);
    let form_area = Rect::new(x, y, form_width, form_height);

    let block = Block::default()
        .title(format!(" {} ", title))
        .title_style(
            Style::default()
                .fg(theme::MAUVE)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::MAUVE))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(form_area);
    frame.render_widget(Clear, form_area);
    frame.render_widget(block, form_area);
    frame.render_widget(Paragraph::new(lines), inner);
}

pub fn render_confirm_popup(frame: &mut Frame, area: Rect, message: &str, hint: &str) {
    let popup_width = (area.width as usize).clamp(40, 60) as u16;
    let popup_height = 7u16;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    let block = Block::default()
        .title(" Confirm ")
        .title_style(
            Style::default()
                .fg(theme::YELLOW)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::YELLOW))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(popup_area);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);

    let lines = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            message.to_string(),
            Style::default()
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD),
        ))),
        lines[0],
    );

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            hint.to_string(),
            Style::default().fg(theme::SUBTEXT),
        ))),
        lines[1],
    );
}

pub fn render_problem_detail(frame: &mut Frame, area: Rect, problem: &Problem) {
    let popup_width = (area.width as usize).clamp(50, 80) as u16;
    let popup_height = (area.height as usize).clamp(15, 30) as u16;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    let block = Block::default()
        .title(format!(" {} ", problem.name))
        .title_style(
            Style::default()
                .fg(theme::BLUE)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BLUE))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(popup_area);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(2),
        ])
        .split(inner);

    let diff_color = match problem.difficulty {
        devcore_challenges::Difficulty::Easy => theme::GREEN,
        devcore_challenges::Difficulty::Medium => theme::YELLOW,
        devcore_challenges::Difficulty::Hard => theme::RED,
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("[{}] ", problem.difficulty),
            Style::default()
                .fg(diff_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{} ", problem.category),
            Style::default().fg(theme::MAUVE),
        ),
        Span::styled(
            problem.tags.join(", ").to_string(),
            Style::default().fg(theme::SUBTEXT),
        ),
    ]));
    frame.render_widget(header, chunks[0]);

    let desc = Paragraph::new(problem.description.as_str())
        .style(Style::default().fg(theme::TEXT))
        .wrap(Wrap { trim: false });
    frame.render_widget(desc, chunks[1]);

    if !problem.hints.is_empty() {
        let hints: Vec<Line> = problem
            .hints
            .iter()
            .enumerate()
            .map(|(i, h)| {
                Line::from(vec![
                    Span::styled(
                        format!("Hint {}: ", i + 1),
                        Style::default()
                            .fg(theme::YELLOW)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(h.as_str(), Style::default().fg(theme::SUBTEXT)),
                ])
            })
            .collect();
        let hints_block = Block::default()
            .title(" Hints ")
            .title_style(
                Style::default()
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY));
        frame.render_widget(List::new(hints).block(hints_block), chunks[3]);
    }

    if !problem.test_cases.is_empty() {
        let test_rows: Vec<Row> = problem
            .test_cases
            .iter()
            .enumerate()
            .map(|(i, tc)| {
                Row::new(vec![
                    Cell::from(Span::styled(
                        format!("{}", i + 1),
                        Style::default().fg(theme::SUBTEXT),
                    )),
                    Cell::from(Span::styled(
                        tc.input.clone(),
                        Style::default().fg(theme::TEXT),
                    )),
                    Cell::from(Span::styled(
                        tc.expected.clone(),
                        Style::default().fg(theme::GREEN),
                    )),
                ])
            })
            .collect();
        let test_block = Block::default()
            .title(" Test Cases ")
            .title_style(
                Style::default()
                    .fg(theme::TEAL)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY));
        let test_table = Table::new(
            test_rows,
            [
                Constraint::Length(4),
                Constraint::Percentage(45),
                Constraint::Percentage(45),
            ],
        )
        .header(
            Row::new(vec![
                Cell::from(Span::styled(
                    "#",
                    Style::default()
                        .fg(theme::MAUVE)
                        .add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    "Input",
                    Style::default()
                        .fg(theme::MAUVE)
                        .add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    "Expected",
                    Style::default()
                        .fg(theme::MAUVE)
                        .add_modifier(Modifier::BOLD),
                )),
            ]),
        );
        frame.render_widget(test_table.block(test_block), chunks[4]);
    }

    let footer = Paragraph::new(Line::from(Span::styled(
        " [a] attempt | [h] hint | Esc close ",
        Style::default().fg(theme::SUBTEXT),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

pub fn render_project_detail(frame: &mut Frame, area: Rect, project: &ProjectPack) {
    let popup_width = (area.width as usize).clamp(50, 90) as u16;
    let popup_height = (area.height as usize).clamp(15, 35) as u16;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    let block = Block::default()
        .title(format!(" {} ", project.name))
        .title_style(
            Style::default()
                .fg(theme::BLUE)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BLUE))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(popup_area);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Min(2),
        ])
        .split(inner);

    let diff_color = match project.difficulty.as_str() {
        "easy" => theme::GREEN,
        "medium" => theme::YELLOW,
        "hard" => theme::RED,
        _ => theme::SUBTEXT,
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("[{}] ", project.difficulty),
            Style::default()
                .fg(diff_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{} ", project.language),
            Style::default().fg(theme::MAUVE),
        ),
        Span::styled(
            format!("{} stages", project.stages.len()),
            Style::default().fg(theme::SUBTEXT),
        ),
    ]));
    frame.render_widget(header, chunks[0]);

    let desc = Paragraph::new(project.description.as_str())
        .style(Style::default().fg(theme::TEXT))
        .wrap(Wrap { trim: false });
    frame.render_widget(desc, chunks[1]);

    let footer = Paragraph::new(Line::from(Span::styled(
        " Press Esc to close ",
        Style::default().fg(theme::SUBTEXT),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);

    if !project.readme.is_empty() {
        let readme_text = if project.readme.chars().count() > 500 {
            let truncated: String = project.readme.chars().take(500).collect();
            format!("{}...", truncated)
        } else {
            project.readme.clone()
        };
        let readme_block = Block::default()
            .title(" Readme ")
            .title_style(
                Style::default()
                    .fg(theme::TEAL)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::OVERLAY));
        let readme = Paragraph::new(readme_text.as_str())
            .style(Style::default().fg(theme::TEXT))
            .wrap(Wrap { trim: false });
        frame.render_widget(readme.block(readme_block), chunks[3]);
    }
}

pub(crate) fn render_add_xp_popup(
    frame: &mut Frame,
    area: Rect,
    axes: &[&str],
    selected_axis: usize,
    amount: &str,
    reason: &str,
    current_field: XpField,
) {
    let popup_width = (area.width as usize).clamp(35, 50) as u16;
    let popup_height = 10u16;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    let block = Block::default()
        .title(" Add XP ")
        .title_style(
            Style::default()
                .fg(theme::GREEN)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::GREEN))
        .style(Style::default().bg(theme::SURFACE));

    let inner = block.inner(popup_area);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(inner);

    let axis_style = if current_field == XpField::Axis {
        Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::TEXT)
    };

    let axis_display = axes.get(selected_axis).unwrap_or(&"?");
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Axis: ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(format!("{} ", axis_display), axis_style),
            Span::styled("<Up/Down>", Style::default().fg(theme::OVERLAY)),
        ])),
        chunks[0],
    );

    let amount_style = if current_field == XpField::Amount {
        Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::TEXT)
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("XP:   ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(format!("{}_", amount), amount_style),
        ])),
        chunks[2],
    );

    let reason_style = if current_field == XpField::Reason {
        Style::default().fg(theme::YELLOW).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::TEXT)
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Why:  ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(format!("{}_", reason), reason_style),
        ])),
        chunks[4],
    );

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " Tab:next field | Enter:confirm | Esc:cancel ",
            Style::default().fg(theme::SUBTEXT),
        )))
        .alignment(Alignment::Center),
        chunks[5],
    );
}
