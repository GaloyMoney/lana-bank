#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod build_info;
pub mod config;
mod db;

use anyhow::Context;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, aead::OsRng};
use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf};

pub use self::build_info::BuildInfo;
use self::config::{Config, EnvSecrets};

#[derive(Parser)]
#[clap(long_about = None)]
struct Cli {
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
    #[clap(env = "SMTP_USERNAME", default_value = "")]
    smtp_username: String,
    #[clap(env = "SMTP_PASSWORD", default_value = "")]
    smtp_password: String,
    #[clap(long, env = "ENCRYPTION_KEY", default_value = "")]
    encryption_key: String,
    #[clap(long, env = "LANA_HOME", default_value = ".lana")]
    lana_home: String,
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show build information including compilation flags
    BuildInfo,
    /// Generate encryption key
    Genencryptionkey,
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
        Commands::Genencryptionkey => {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            println!("{}", hex::encode(key));
            return Ok(());
        }
        Commands::Run => {
            let config = Config::init(
                cli.config,
                EnvSecrets {
                    pg_con: cli.pg_con,
                    sumsub_key: cli.sumsub_key,
                    sumsub_secret: cli.sumsub_secret,
                    smtp_username: cli.smtp_username,
                    smtp_password: cli.smtp_password,
                    encryption_key: cli.encryption_key,
                },
            )?;

            run_cmd(&cli.lana_home, config).await?;
        }
    }

    Ok(())
}

async fn run_cmd(lana_home: &str, config: Config) -> anyhow::Result<()> {
    tracing_utils::init_tracer(config.tracing)?;
    store_server_pid(lana_home, std::process::id())?;

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

pub fn store_server_pid(lana_home: &str, pid: u32) -> anyhow::Result<()> {
    create_lana_dir(lana_home)?;
    let _ = fs::remove_file(format!("{lana_home}/server-pid"));
    fs::write(format!("{lana_home}/server-pid"), pid.to_string()).context("Writing PID file")?;
    Ok(())
}

fn create_lana_dir(lana_home: &str) -> anyhow::Result<()> {
    let _ = fs::create_dir(lana_home);
    Ok(())
}
