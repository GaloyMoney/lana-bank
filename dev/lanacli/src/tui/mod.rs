mod app;
mod data;
mod views;

use std::io;
use std::sync::Arc;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::{Mutex, mpsc};

use crate::client::GraphQLClient;
use app::{App, AsyncResult, PendingAction, Screen};

pub async fn run(client: GraphQLClient) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    let result = run_app(&mut terminal, client).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    client: GraphQLClient,
) -> anyhow::Result<()> {
    let client = Arc::new(Mutex::new(client));
    let (tx, mut rx) = mpsc::unbounded_channel::<AsyncResult>();
    let mut app = App::new();

    loop {
        terminal.draw(|f| views::render(f, &mut app))?;

        while let Ok(result) = rx.try_recv() {
            app.handle_async_result(result);
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if app.input_mode {
                    match key.code {
                        KeyCode::Enter => {
                            let reason: String = app.input_buffer.drain(..).collect();
                            app.input_mode = false;
                            if let Some(pending) = app.pending_action.take() {
                                app.loading = true;
                                app.status = "Submitting...".into();
                                let cl = client.clone();
                                let tx = tx.clone();
                                let eid = pending.entity_id;
                                let domain = pending.domain;
                                let action = pending.action;
                                tokio::spawn(async move {
                                    let result = data::execute_action(
                                        &cl,
                                        domain,
                                        &eid,
                                        action,
                                        Some(reason),
                                    )
                                    .await;
                                    let _ = tx.send(AsyncResult::ActionDone(result));
                                });
                            }
                        }
                        KeyCode::Esc => {
                            app.input_mode = false;
                            app.input_buffer.clear();
                            app.pending_action = None;
                            app.status = "Cancelled".into();
                        }
                        KeyCode::Char(c) => app.input_buffer.push(c),
                        KeyCode::Backspace => {
                            app.input_buffer.pop();
                        }
                        _ => {}
                    }
                    continue;
                }

                match app.screen {
                    Screen::DomainMenu => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Up | KeyCode::Char('k') => {
                            let i = app.menu_state.selected().unwrap_or(0);
                            let count = app::ALL_DOMAINS.len();
                            let next = if i == 0 { count - 1 } else { i - 1 };
                            app.menu_state.select(Some(next));
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let i = app.menu_state.selected().unwrap_or(0);
                            let count = app::ALL_DOMAINS.len();
                            let next = (i + 1) % count;
                            app.menu_state.select(Some(next));
                        }
                        KeyCode::Enter => {
                            let domain = app.selected_domain();
                            app.loading = true;
                            app.status = format!("Loading {}...", domain.label());
                            let cl = client.clone();
                            let tx = tx.clone();
                            tokio::spawn(async move {
                                let result = data::fetch_list(&cl, domain, 25, None).await;
                                let _ = tx.send(AsyncResult::ListLoaded(domain, result));
                            });
                        }
                        _ => {}
                    },
                    Screen::ListView => match key.code {
                        KeyCode::Esc => {
                            app.screen = Screen::DomainMenu;
                            app.status = "Navigate with arrows, Enter to select, q to quit".into();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let i = app.table_state.selected().unwrap_or(0);
                            let count = app.list.rows.len();
                            if count > 0 {
                                let next = if i == 0 { count - 1 } else { i - 1 };
                                app.table_state.select(Some(next));
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let i = app.table_state.selected().unwrap_or(0);
                            let count = app.list.rows.len();
                            if count > 0 {
                                let next = (i + 1) % count;
                                app.table_state.select(Some(next));
                            }
                        }
                        KeyCode::Enter => {
                            if let (Some(domain), Some(selected)) =
                                (app.current_domain, app.table_state.selected())
                            {
                                if selected < app.list.ids.len() {
                                    if domain.has_detail_query() {
                                        let id = app.list.ids[selected].clone();
                                        app.loading = true;
                                        app.status = "Loading detail...".into();
                                        let cl = client.clone();
                                        let tx = tx.clone();
                                        tokio::spawn(async move {
                                            let result = data::fetch_detail(&cl, domain, &id).await;
                                            let _ = tx.send(AsyncResult::DetailLoaded(result));
                                        });
                                    } else {
                                        app.enter_detail_from_list_row();
                                    }
                                }
                            }
                        }
                        KeyCode::Char('n') => {
                            if let Some(domain) = app.current_domain {
                                if app.list.has_next_page {
                                    let cursor = app.list.end_cursor.clone();
                                    app.loading = true;
                                    app.status = "Loading next page...".into();
                                    let cl = client.clone();
                                    let tx = tx.clone();
                                    tokio::spawn(async move {
                                        let result =
                                            data::fetch_list(&cl, domain, 25, cursor).await;
                                        let _ = tx.send(AsyncResult::ListLoaded(domain, result));
                                    });
                                }
                            }
                        }
                        KeyCode::Char('r') => {
                            if let Some(domain) = app.current_domain {
                                app.loading = true;
                                app.status = "Refreshing...".into();
                                let cl = client.clone();
                                let tx = tx.clone();
                                tokio::spawn(async move {
                                    let result = data::fetch_list(&cl, domain, 25, None).await;
                                    let _ = tx.send(AsyncResult::ListLoaded(domain, result));
                                });
                            }
                        }
                        _ => {}
                    },
                    Screen::DetailView => match key.code {
                        KeyCode::Esc => {
                            app.screen = Screen::ListView;
                            if let Some(domain) = app.current_domain {
                                let count = app.list.rows.len();
                                let more = if app.list.has_next_page {
                                    " (more available, press n)"
                                } else {
                                    ""
                                };
                                app.status = format!(
                                    "{count} {}{more} | Enter=detail  r=refresh  Esc=back",
                                    domain.label()
                                );
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.detail_scroll = app.detail_scroll.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.detail_scroll = app.detail_scroll.saturating_add(1);
                        }
                        KeyCode::Char(c) => {
                            if let Some(action) =
                                app.detail.actions.iter().find(|a| a.key() == c).copied()
                            {
                                if action.needs_input() {
                                    app.input_mode = true;
                                    app.input_buffer.clear();
                                    app.pending_action = Some(PendingAction {
                                        action,
                                        entity_id: app.detail.entity_id.clone(),
                                        domain: app.current_domain.unwrap(),
                                    });
                                } else {
                                    app.loading = true;
                                    app.status = "Executing action...".into();
                                    let cl = client.clone();
                                    let tx = tx.clone();
                                    let eid = app.detail.entity_id.clone();
                                    let domain = app.current_domain.unwrap();
                                    tokio::spawn(async move {
                                        let result =
                                            data::execute_action(&cl, domain, &eid, action, None)
                                                .await;
                                        let _ = tx.send(AsyncResult::ActionDone(result));
                                    });
                                }
                            }
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}
