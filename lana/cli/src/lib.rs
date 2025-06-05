#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod build_info;
pub mod config;
mod db;

use anyhow::Context;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub use self::build_info::BuildInfo;
use self::config::{Config, EnvSecrets};

#[derive(Parser)]
#[clap(long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,

    #[clap(
        short,
        long,
        env = "LANA_CONFIG",
        default_value = "lana.yml",
        value_name = "FILE"
    )]
    config: PathBuf,
    #[clap(env = "PG_CON")]
    pg_con: String,
    #[clap(env = "SUMSUB_KEY", default_value = "")]
    sumsub_key: String,
    #[clap(env = "SUMSUB_SECRET", default_value = "")]
    sumsub_secret: String,
    #[clap(env = "SA_CREDS_BASE64", default_value = "")]
    sa_creds_base64_raw: String,
    #[clap(env = "DEV_ENV_NAME_PREFIX")]
    dev_env_name_prefix: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show build information including compilation flags
    BuildInfo,
    /// Run the main server (default when no subcommand is specified)
    Run,
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Run) {
        Commands::BuildInfo => {
            let build_info = BuildInfo::get();
            println!("{}", build_info.display());
            return Ok(());
        }
        Commands::Run => {
            // Continue with existing server logic
        }
    }

    let sa_creds_base64 = if cli.sa_creds_base64_raw.is_empty() {
        None
    } else {
        Some(cli.sa_creds_base64_raw)
    };

    let config = Config::init(
        cli.config,
        EnvSecrets {
            pg_con: cli.pg_con,
            sumsub_key: cli.sumsub_key,
            sumsub_secret: cli.sumsub_secret,
            sa_creds_base64,
        },
        cli.dev_env_name_prefix,
    )?;

    run_cmd(config).await?;

    Ok(())
}

async fn run_cmd(config: Config) -> anyhow::Result<()> {
    tracing_utils::init_tracer(config.tracing)?;

    #[cfg(feature = "sim-time")]
    {
        sim_time::init(config.time);
    }

    let (send, mut receive) = tokio::sync::mpsc::channel(1);
    let mut handles = Vec::new();
    let pool = db::init_pool(&config.db).await?;

    #[cfg(feature = "sim-bootstrap")]
    let superuser_email = config
        .app
        .access
        .superuser_email
        .clone()
        .expect("super user");

    let admin_app = lana_app::app::LanaApp::run(pool.clone(), config.app).await?;
    let customer_app = admin_app.clone();

    #[cfg(feature = "sim-bootstrap")]
    {
        let _ = sim_bootstrap::run(superuser_email.to_string(), &admin_app, config.bootstrap).await;
    }

    let admin_send = send.clone();

    handles.push(tokio::spawn(async move {
        let _ = admin_send.try_send(
            admin_server::run(config.admin_server, admin_app)
                .await
                .context("Admin server error"),
        );
    }));
    let customer_send = send.clone();
    handles.push(tokio::spawn(async move {
        let _ = customer_send.try_send(
            customer_server::run(config.customer_server, customer_app)
                .await
                .context("Customer server error"),
        );
    }));

    let reason = receive.recv().await.expect("Didn't receive msg");
    for handle in handles {
        handle.abort();
    }

    reason
}
