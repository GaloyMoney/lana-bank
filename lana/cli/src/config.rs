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
use customer_server::CustomerServerConfig;
use lana_app::{app::AppConfig, storage::config::StorageConfig};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub db: DbConfig,
    #[serde(default)]
    pub admin_server: AdminServerConfig,
    #[serde(default)]
    pub customer_server: CustomerServerConfig,
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
    pub smtp_username: String,
    pub smtp_password: String,
    pub encryption_key: String,
    pub keycloak_internal_client_secret: String,
    pub keycloak_customer_client_secret: String,
}

impl Config {
    pub fn init(
        path: impl AsRef<Path>,
        EnvSecrets {
            pg_con,
            sumsub_key,
            sumsub_secret,
            smtp_username,
            smtp_password,
            encryption_key,
            keycloak_internal_client_secret,
            keycloak_customer_client_secret,
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
        config.app.notification.email.username = smtp_username;
        config.app.notification.email.password = smtp_password;
        config.app.user_onboarding.keycloak.client_secret = keycloak_internal_client_secret;
        config.app.customer_sync.keycloak.client_secret = keycloak_customer_client_secret;
        if let Some(dev_env_name_prefix) = dev_env_name_prefix {
            eprintln!(
                "WARNING - overriding GCP-related config from DEV_ENV_NAME_PREFIX={dev_env_name_prefix}"
            );
            if config.app.storage.identifier().contains("gcp") {
                config.app.storage = StorageConfig::new_gcp_dev_mode(dev_env_name_prefix);
            }
        }

        let key_bytes = hex::decode(encryption_key)?;
        if key_bytes.len() != 32 {
            return Err(anyhow::anyhow!(
                "Custodian encryption key must be 32 bytes, got {}",
                key_bytes.len()
            ));
        }

        config.app.custody.encryption.key =
            chacha20poly1305::Key::clone_from_slice(key_bytes.as_ref());

        Ok(config)
    }
}
