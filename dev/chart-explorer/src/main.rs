mod app;
mod db;
mod ui;

use std::collections::HashMap;
use std::io;

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use sqlx::postgres::PgPoolOptions;

#[derive(Parser)]
#[command(
    name = "chart-explorer",
    about = "Explore LANA chart and CALA account set DAGs"
)]
struct Cli {
    /// Print both trees as indented text to stdout (no TUI)
    #[arg(long)]
    dump: bool,

    /// Print both trees as JSON to stdout (no TUI)
    #[arg(long)]
    dump_json: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let pg_con = std::env::var("PG_CON")
        .unwrap_or_else(|_| "postgres://user:password@localhost:5433/pg".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&pg_con)
        .await?;

    // Load all data
    let charts = db::load_charts(&pool).await?;
    let mut chart_nodes = HashMap::new();
    for chart in &charts {
        let nodes = db::load_chart_nodes(&pool, chart.id).await?;
        chart_nodes.insert(chart.id, nodes);
    }
    let cala_sets = db::load_account_sets(&pool).await?;
    let cala_set_members = db::load_set_member_sets(&pool).await?;
    let cala_account_members = db::load_set_member_accounts(&pool).await?;

    let app = app::App::new(
        charts,
        chart_nodes,
        cala_sets,
        cala_set_members,
        cala_account_members,
    );

    if cli.dump {
        app::dump_text(&app);
        return Ok(());
    }

    if cli.dump_json {
        app::dump_json(&app);
        return Ok(());
    }

    run_tui(app)?;

    Ok(())
}

fn run_tui(mut app: app::App) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Tab => app.toggle_view(),
                KeyCode::Up => match app.active_view {
                    app::ActiveView::Lana => {
                        app.lana_tree_state.key_up();
                    }
                    app::ActiveView::Cala => {
                        app.cala_tree_state.key_up();
                    }
                },
                KeyCode::Down => match app.active_view {
                    app::ActiveView::Lana => {
                        app.lana_tree_state.key_down();
                    }
                    app::ActiveView::Cala => {
                        app.cala_tree_state.key_down();
                    }
                },
                KeyCode::Left => match app.active_view {
                    app::ActiveView::Lana => {
                        app.lana_tree_state.key_left();
                    }
                    app::ActiveView::Cala => {
                        app.cala_tree_state.key_left();
                    }
                },
                KeyCode::Right => match app.active_view {
                    app::ActiveView::Lana => {
                        app.lana_tree_state.key_right();
                    }
                    app::ActiveView::Cala => {
                        app.cala_tree_state.key_right();
                    }
                },
                KeyCode::Enter => match app.active_view {
                    app::ActiveView::Lana => {
                        app.lana_tree_state.toggle_selected();
                    }
                    app::ActiveView::Cala => {
                        app.cala_tree_state.toggle_selected();
                    }
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
