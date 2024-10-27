pub mod config;
mod db;

use anyhow::Context;
use clap::Parser;
use std::{fs, path::PathBuf};

use self::config::{Config, EnvSecrets};

#[derive(Parser)]
#[clap(long_about = None)]
struct Cli {
    #[clap(
        short,
        long,
        env = "LAVA_CONFIG",
        default_value = "lava.yml",
        value_name = "FILE"
    )]
    config: PathBuf,
    #[clap(
        long,
        env = "LAVA_HOME",
        default_value = ".lava",
        value_name = "DIRECTORY"
    )]
    lava_home: String,
    #[clap(env = "PG_CON")]
    pg_con: String,
    #[clap(env = "SUMSUB_KEY", default_value = "")]
    sumsub_key: String,
    #[clap(env = "SUMSUB_SECRET", default_value = "")]
    sumsub_secret: String,
    #[clap(env = "SA_CREDS_BASE64", default_value = "")]
    sa_creds_base64: String,
    #[clap(env = "DEV_ENV_NAME_PREFIX")]
    dev_env_name_prefix: Option<String>,
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = Config::init(
        cli.config,
        EnvSecrets {
            pg_con: cli.pg_con,
            sumsub_key: cli.sumsub_key,
            sumsub_secret: cli.sumsub_secret,
            sa_creds_base64: cli.sa_creds_base64,
        },
        cli.dev_env_name_prefix,
    )?;

    run_cmd(&cli.lava_home, config).await?;

    Ok(())
}

async fn run_cmd(lava_home: &str, config: Config) -> anyhow::Result<()> {
    tracing_utils::init_tracer(config.tracing)?;
    store_server_pid(lava_home, std::process::id())?;

    let (send, mut receive) = tokio::sync::mpsc::channel(1);
    let mut handles = Vec::new();
    let pool = db::init_pool(&config.db).await?;
    let public_app = crate::app::LavaApp::run(pool.clone(), config.app).await?;
    let admin_app = public_app.clone();

    let admin_send = send.clone();

    handles.push(tokio::spawn(async move {
        let _ = admin_send.try_send(
            crate::server::admin::run(config.admin_server, admin_app)
                .await
                .context("Admin server error"),
        );
    }));

    let public_send = send.clone();
    handles.push(tokio::spawn(async move {
        let _ = public_send.try_send(
            crate::server::public::run(config.public_server, public_app)
                .await
                .context("Public server error"),
        );
    }));

    let reason = receive.recv().await.expect("Didn't receive msg");
    for handle in handles {
        handle.abort();
    }

    reason
}

pub fn store_server_pid(lava_home: &str, pid: u32) -> anyhow::Result<()> {
    create_lava_dir(lava_home)?;
    let _ = fs::remove_file(format!("{lava_home}/server-pid"));
    fs::write(format!("{lava_home}/server-pid"), pid.to_string()).context("Writing PID file")?;
    Ok(())
}

fn create_lava_dir(lava_home: &str) -> anyhow::Result<()> {
    let _ = fs::create_dir(lava_home);
    Ok(())
}
