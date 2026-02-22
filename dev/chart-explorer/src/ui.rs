use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use tui_tree_widget::Tree;

use crate::app::{ActiveView, App};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    let main_area = chunks[0];
    let status_bar = chunks[1];

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(main_area);

    draw_tree_panel(f, app, main_chunks[0]);
    draw_details_panel(f, app, main_chunks[1]);
    draw_status_bar(f, app, status_bar);
}

fn draw_tree_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let highlight_style = Style::default()
        .fg(Color::Black)
        .bg(Color::LightGreen)
        .add_modifier(Modifier::BOLD);

    match app.active_view {
        ActiveView::Lana => {
            let block = Block::default()
                .title(" LANA Chart Tree ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green));

            let tree = Tree::new(&app.lana_items)
                .expect("valid tree")
                .block(block)
                .highlight_style(highlight_style)
                .highlight_symbol("▸ ")
                .node_closed_symbol("▶ ")
                .node_open_symbol("▼ ")
                .node_no_children_symbol("  ");

            f.render_stateful_widget(tree, area, &mut app.lana_tree_state);
        }
        ActiveView::Cala => {
            let block = Block::default()
                .title(" CALA Account Sets ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));

            let tree = Tree::new(&app.cala_items)
                .expect("valid tree")
                .block(block)
                .highlight_style(highlight_style)
                .highlight_symbol("▸ ")
                .node_closed_symbol("▶ ")
                .node_open_symbol("▼ ")
                .node_no_children_symbol("  ");

            f.render_stateful_widget(tree, area, &mut app.cala_tree_state);
        }
    }
}

fn draw_details_panel(f: &mut Frame, app: &App, area: Rect) {
    let details = app.selected_details();
    let lines: Vec<Line> = details.iter().map(|s| Line::from(s.as_str())).collect();

    let block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let lana_style = if app.active_view == ActiveView::Lana {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    let cala_style = if app.active_view == ActiveView::Cala {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let jump_hint = if app.can_jump() {
        Span::styled(
            "  g: jump to counterpart",
            Style::default().fg(Color::Yellow),
        )
    } else {
        Span::raw("")
    };

    let line = Line::from(vec![
        Span::styled(" [LANA Tree] ", lana_style),
        Span::raw(" "),
        Span::styled(" [CALA Sets] ", cala_style),
        Span::raw("  Tab: switch  ↑↓: navigate  ←→: collapse/expand  q: quit"),
        jump_hint,
    ]);

    f.render_widget(Paragraph::new(line), area);
}
