use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::tui::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let domain_label = match app.current_domain {
        Some(d) => d.label(),
        None => "Detail",
    };

    let max_key_len = app
        .detail
        .pairs
        .iter()
        .map(|(k, _)| k.len())
        .max()
        .unwrap_or(0);

    let lines: Vec<Line> = app
        .detail
        .pairs
        .iter()
        .map(|(key, val)| {
            Line::from(vec![
                Span::styled(
                    format!("{:>width$}", key, width = max_key_len),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(" : "),
                Span::raw(val.as_str()),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} — {} ", domain_label, app.detail.entity_id)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));

    frame.render_widget(paragraph, area);
}
