use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::theme;
use devcore_academic::UrgencyLevel;

pub struct KeyBinding {
    pub key: &'static str,
    pub label: &'static str,
    pub color: Color,
}

pub fn status_bar(frame: &mut Frame, area: Rect, keybindings: &[KeyBinding]) {
    let mut spans: Vec<Span> = Vec::new();
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
        UrgencyLevel::Overdue => "[OVRD]".to_string(),
        UrgencyLevel::Critical => "[!!!]".to_string(),
        UrgencyLevel::Warning => " [!!]".to_string(),
        UrgencyLevel::Soon => "  [!]".to_string(),
        UrgencyLevel::Normal => "     ".to_string(),
    }
}
