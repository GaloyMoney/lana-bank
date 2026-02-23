use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
};

use crate::tui::app::{ALL_DOMAINS, App};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = ALL_DOMAINS
        .iter()
        .map(|d| ListItem::new(format!("  {}", d.label())))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" LANA Admin — Select Domain "),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(list, area, &mut app.menu_state);
}
