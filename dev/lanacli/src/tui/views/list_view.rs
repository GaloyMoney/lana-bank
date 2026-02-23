use ratatui::{
    Frame,
    layout::Constraint,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::tui::app::App;

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let domain_label = match app.current_domain {
        Some(d) => d.label(),
        None => "Unknown",
    };

    let header_cells: Vec<Cell> = app
        .list
        .headers
        .iter()
        .map(|h| Cell::from(h.as_str()).style(Style::default().fg(Color::Yellow)))
        .collect();
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .list
        .rows
        .iter()
        .map(|row| {
            let cells: Vec<Cell> = row.iter().map(|c| Cell::from(c.as_str())).collect();
            Row::new(cells)
        })
        .collect();

    let col_count = app.list.headers.len().max(1) as u16;
    let widths: Vec<Constraint> = (0..col_count).map(|_| Constraint::Fill(1)).collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", domain_label)),
        )
        .row_highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(table, area, &mut app.table_state);
}
