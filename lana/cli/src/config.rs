use anyhow::Context;

use serde::{Deserialize, Serialize};
use tracing_utils::TracingConfig;

#[cfg(feature = "sim-time")]
use sim_time::TimeConfig;

#[cfg(feature = "sim-bootstrap")]
use sim_bootstrap::BootstrapConfig;

use std::path::Path;

use super::db::*;
use admin_server::AdminServerConfig;
use lana_app::{app::AppConfig, report::ReportConfig, storage::config::StorageConfig};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub db: DbConfig,
    #[serde(default)]
    pub admin_server: AdminServerConfig,
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub tracing: TracingConfig,

    #[cfg(feature = "sim-time")]
    #[serde(default)]
    pub time: TimeConfig,
    #[cfg(feature = "sim-bootstrap")]
    #[serde(default)]
    pub bootstrap: BootstrapConfig,
}

pub struct EnvSecrets {
    pub pg_con: String,
    pub sumsub_key: String,
    pub sumsub_secret: String,
    pub sa_creds_base64: String,
}

impl Config {
    pub fn init(
        path: impl AsRef<Path>,
        EnvSecrets {
            pg_con,
            sumsub_key,
            sumsub_secret,
            sa_creds_base64,
        }: EnvSecrets,
        dev_env_name_prefix: Option<String>,
    ) -> anyhow::Result<Self> {
        let config_file = std::fs::read_to_string(&path)
            .context(format!("Couldn't read config file {:?}", path.as_ref()))?;

        let mut config: Config =
            serde_yaml::from_str(&config_file).context("Couldn't parse config file")?;

        config.db.pg_con.clone_from(&pg_con);
        config.app.sumsub.sumsub_key = sumsub_key;
        config.app.sumsub.sumsub_secret = sumsub_secret;
        config.app.service_account = config
            .app
            .service_account
            .set_sa_creds_base64(sa_creds_base64)?;
        if let Some(dev_env_name_prefix) = dev_env_name_prefix {
            eprintln!(
                "WARNING - overriding GCP-related config from DEV_ENV_NAME_PREFIX={}",
                dev_env_name_prefix
            );
            config.app.report = ReportConfig::new_dev_mode(
                dev_env_name_prefix.clone(),
                config.app.service_account.clone(),
            );
            config.app.storage = StorageConfig::new_dev_mode(
                dev_env_name_prefix,
                config.app.service_account.clone(),
            );
        } else {
            config.app.report.service_account = Some(config.app.service_account.clone());
            config.app.storage.service_account = Some(config.app.service_account.clone());
        };

        Ok(config)
    }
}
