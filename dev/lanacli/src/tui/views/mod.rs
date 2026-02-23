mod detail_view;
mod domain_menu;
mod list_view;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::app::{App, Screen};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(frame.area());

    let main_area = chunks[0];
    let status_area = chunks[1];

    match app.screen {
        Screen::DomainMenu => domain_menu::render(frame, app, main_area),
        Screen::ListView => list_view::render(frame, app, main_area),
        Screen::DetailView => detail_view::render(frame, app, main_area),
    }

    render_status_bar(frame, app, status_area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let content = if app.loading {
        Line::from(vec![
            Span::styled(
                " Loading... ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&app.status),
        ])
    } else if app.input_mode {
        Line::from(vec![
            Span::styled(" Deny reason: ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.input_buffer),
            Span::styled("█", Style::default().fg(Color::White)),
        ])
    } else {
        Line::from(Span::raw(format!(" {}", &app.status)))
    };

    let status = Paragraph::new(content).block(Block::default().borders(Borders::ALL));
    frame.render_widget(status, area);
}
