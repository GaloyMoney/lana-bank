#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
mod db;

use anyhow::Context;
use chacha20poly1305::{aead::OsRng, ChaCha20Poly1305, KeyInit};
use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf};

use self::config::{Config, EnvSecrets};

#[derive(Parser)]
#[clap(long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Option<UtilsCommands>,

    #[clap(
        long,
        env = "LANA_HOME",
        default_value = ".lana",
        value_name = "DIRECTORY"
    )]
    lana_home: String,
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
    #[clap(long, env = "CUSTODIAN_ENCRYPTION_KEY", default_value = "")]
    custodian_encryption_key: String,
}

#[derive(Subcommand)]
enum UtilsCommands {
    Genencryptionkey,
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(UtilsCommands::Genencryptionkey) => {
            let key = ChaCha20Poly1305::generate_key(&mut OsRng);
            println!("{}", hex::encode(key));
        }
        None => {
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
                    custodian_encryption_key: cli.custodian_encryption_key,
                },
                cli.dev_env_name_prefix,
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
