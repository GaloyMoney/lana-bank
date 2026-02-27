#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod build_info;
pub(crate) mod config;
mod db;
mod startup_domain_config;

use anyhow::Context;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, aead::OsRng};
use clap::{Parser, Subcommand};
use std::{ffi::OsString, fs, path::PathBuf, time::Duration};
use tracing_utils::{error, info, warn};

pub use self::build_info::BuildInfo;
use self::config::{Config, EnvSecrets};

#[derive(Parser)]
#[clap(version = env!("BUILD_VERSION"), long_about = None)]
struct Cli {
    #[clap(
        short,
        long,
        env = "LANA_CONFIG",
        default_value = "lana.yml",
        value_name = "FILE"
    )]
    config: PathBuf,
    #[clap(long, env = "PG_CON", default_value = "")]
    pg_con: String,
    #[clap(long, env = "SMTP_USERNAME", default_value = "")]
    smtp_username: String,
    #[clap(long, env = "SMTP_PASSWORD", default_value = "")]
    smtp_password: String,
    #[clap(long, env = "ENCRYPTION_KEY", default_value = "")]
    encryption_key: String,
    #[clap(
        long,
        env = "KEYCLOAK_INTERNAL_CLIENT_SECRET",
        default_value = "secret"
    )]
    keycloak_internal_client_secret: String,
    #[clap(
        long,
        env = "KEYCLOAK_CUSTOMER_CLIENT_SECRET",
        default_value = "secret"
    )]
    keycloak_customer_client_secret: String,
    #[clap(long, env = "LANA_HOME", default_value = ".lana")]
    lana_home: String,
    /// Admin GraphQL endpoint URL for CLI admin operations
    #[clap(
        long,
        env = "LANA_ADMIN_URL",
        default_value = "http://admin.localhost:4455/graphql"
    )]
    admin_url: String,
    /// Keycloak URL for CLI admin authentication
    #[clap(
        long,
        env = "LANA_KEYCLOAK_URL",
        default_value = "http://localhost:8081"
    )]
    keycloak_url: String,
    /// Admin username (Keycloak email)
    #[clap(long, env = "LANA_USERNAME", default_value = "admin@galoy.io")]
    username: String,
    /// Admin password
    #[clap(long, env = "LANA_PASSWORD", default_value = "")]
    password: String,
    /// Output admin command results as JSON instead of tables
    #[clap(long, global = true)]
    json: bool,
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Show build information including compilation flags
    BuildInfo,
    /// Generate encryption key
    Genencryptionkey,
    /// Generate default configuration file (lana.yml) with all default values
    DumpDefaultConfig,
    /// Run the main server (default when no subcommand is specified)
    #[command(alias = "run")]
    Serve,
    #[command(external_subcommand)]
    Admin(Vec<String>),
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command.clone().unwrap_or(Commands::Serve) {
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
        Commands::DumpDefaultConfig => {
            let default_config = Config::default();
            let yaml_output = serde_yaml::to_string(&default_config)?;
            println!("{yaml_output}");
            return Ok(());
        }
        Commands::Serve => run_server(&cli).await?,
        Commands::Admin(admin_cmd) => run_admin_command(&cli, admin_cmd).await?,
    }

    Ok(())
}

async fn run_server(cli: &Cli) -> anyhow::Result<()> {
    let config = Config::try_new(
        cli.config.clone(),
        EnvSecrets {
            pg_con: cli.pg_con.clone(),
            smtp_username: cli.smtp_username.clone(),
            smtp_password: cli.smtp_password.clone(),
            encryption_key: cli.encryption_key.clone(),
            keycloak_internal_client_secret: cli.keycloak_internal_client_secret.clone(),
            keycloak_customer_client_secret: cli.keycloak_customer_client_secret.clone(),
        },
    )?;

    run_cmd(&cli.lana_home, config).await
}

async fn run_admin_command(cli: &Cli, admin_cmd: Vec<String>) -> anyhow::Result<()> {
    let mut args = vec![
        OsString::from("lana-cli"),
        OsString::from("--admin-url"),
        OsString::from(&cli.admin_url),
        OsString::from("--keycloak-url"),
        OsString::from(&cli.keycloak_url),
        OsString::from("--username"),
        OsString::from(&cli.username),
        OsString::from("--password"),
        OsString::from(&cli.password),
    ];
    if cli.json {
        args.push(OsString::from("--json"));
    }
    args.extend(admin_cmd.into_iter().map(OsString::from));

    lana_admin_cli::run_from_args(args).await
}

/// Setup GCP credentials by decoding SA_CREDS_BASE64 and setting GOOGLE_APPLICATION_CREDENTIALS
///
/// LONGER-TERM SOLUTION: Replace this with Workload Identity for Kubernetes deployments.
/// Workload Identity is more secure as it eliminates the need for service account key files:
///
/// 1. Configure your GKE cluster with Workload Identity enabled
/// 2. Annotate your Kubernetes service account:
///    ```yaml
///    apiVersion: v1
///    kind: ServiceAccount
///    metadata:
///      annotations:
///        iam.gke.io/gcp-service-account: your-sa@project.iam.gserviceaccount.com
///    ```
/// 3. Remove SA_CREDS_BASE64 environment variable entirely
/// 4. The google-cloud-storage crate will automatically use Workload Identity
///
/// Benefits: No credential files, automatic token rotation, better security posture.
fn setup_gcp_credentials() -> anyhow::Result<()> {
    if let Ok(creds_base64) = std::env::var("SA_CREDS_BASE64") {
        use base64::{Engine as _, engine::general_purpose};

        // Decode the base64-encoded service account JSON
        let creds_bytes = general_purpose::STANDARD
            .decode(creds_base64.as_bytes())
            .context("Failed to decode SA_CREDS_BASE64")?;
        let creds_json = std::str::from_utf8(&creds_bytes)
            .context("Invalid UTF-8 in decoded service account credentials")?;

        // Write credentials to a temporary file
        let creds_path = "/tmp/gcp-service-account.json";
        std::fs::write(creds_path, creds_json)
            .context("Failed to write GCP service account credentials to file")?;

        // Set the environment variable that the google-cloud-storage crate expects
        unsafe {
            std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", creds_path);
        }

        eprintln!("✅ Set up GCP credentials from SA_CREDS_BASE64");
    } else {
        eprintln!("ℹ️  SA_CREDS_BASE64 not set, using default GCP authentication");
    }

    Ok(())
}

async fn run_cmd(lana_home: &str, config: Config) -> anyhow::Result<()> {
    tracing_utils::init_tracer(config.tracing)?;
    store_server_pid(lana_home, std::process::id())?;

    // Setup GCP credentials from SA_CREDS_BASE64 environment variable
    setup_gcp_credentials()?;

    let (error_send, mut error_recv) = tokio::sync::mpsc::channel(1);
    let (shutdown_send, shutdown_recv) = tokio::sync::broadcast::channel(1);
    let mut server_handles = Vec::new();
    let pool = db::init_pool(&config.db).await?;

    #[cfg(feature = "sim-bootstrap")]
    let superuser_email = config
        .app
        .access
        .superuser_email
        .clone()
        .expect("super user");

    let (clock, _clock_ctrl) = config.time.into_clock();

    let domain_config_settings = startup_domain_config::parse_from_env()?;
    let app = lana_app::app::LanaApp::init(
        pool.clone(),
        config.app,
        clock.clone(),
        domain_config_settings.into_iter().map(|s| (s.key, s.value)),
    )
    .await
    .context("Failed to initialize Lana app")?;

    #[cfg(feature = "sim-bootstrap")]
    if let Some(ctrl) = _clock_ctrl {
        let seed_only = config.bootstrap.seed_only;
        let _ = sim_bootstrap::run(
            superuser_email.to_string(),
            &app,
            config.bootstrap,
            clock,
            ctrl,
        )
        .await;
        if seed_only {
            info!("Seed-only mode: bootstrap complete, shutting down");
            if let Err(e) = app.shutdown().await {
                eprintln!("Error shutting down app: {}", e);
            }
            eprintln!("shutdown complete");
            return Ok(());
        }
    }

    let admin_error_send = error_send.clone();
    let admin_app = app.clone();
    let mut admin_shutdown = shutdown_recv.resubscribe();

    server_handles.push(tokio::spawn(async move {
        let _ = admin_error_send.try_send(
            admin_server::run(config.admin_server, admin_app, async move {
                let _ = admin_shutdown.recv().await;
            })
            .await
            .context("Admin server error"),
        );
    }));

    let customer_error_send = error_send.clone();
    let customer_app = app.clone();
    let mut customer_shutdown = shutdown_recv.resubscribe();
    server_handles.push(tokio::spawn(async move {
        let _ = customer_error_send.try_send(
            customer_server::run(config.customer_server, customer_app, async move {
                let _ = customer_shutdown.recv().await;
            })
            .await
            .context("Customer server error"),
        );
    }));

    // Setup signal handlers
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .context("Failed to setup SIGTERM handler")?;
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .context("Failed to setup SIGINT handler")?;

    let result = tokio::select! {
        reason = error_recv.recv() => {
            let reason = reason.expect("Didn't receive error msg");
            if let Err(ref e) = reason {
                error!(error = ?e, "Shutting down due to error");
            } else {
                error!("Shutting down unexpectedly");
            }
            reason
        }
        _ = sigterm.recv() => {
            info!("Received SIGTERM, shutting down gracefully...");
            Ok(())
        }
        _ = sigint.recv() => {
            info!("Received SIGINT (Ctrl-C), shutting down gracefully...");
            Ok(())
        }
    };

    info!("Sending shutdown signal to servers");
    let _ = shutdown_send.send(());

    const SERVER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);
    let shutdown_all = async {
        for handle in server_handles {
            let _ = handle.await;
        }
    };
    if tokio::time::timeout(SERVER_SHUTDOWN_TIMEOUT, shutdown_all)
        .await
        .is_err()
    {
        warn!(
            "Server shutdown timed out after {:?}",
            SERVER_SHUTDOWN_TIMEOUT
        );
    }
    info!("Server handles finished");

    if let Err(e) = app.shutdown().await {
        eprintln!("Error shutting down app: {}", e);
    }
    eprintln!("shutdown complete");

    result
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
