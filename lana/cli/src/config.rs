use anyhow::Context;

use es_entity::clock::{ArtificialClockConfig, ClockController, ClockHandle};
use serde::{Deserialize, Serialize};
use tracing_utils::TracingConfig;

#[cfg(feature = "sim-bootstrap")]
use sim_bootstrap::BootstrapConfig;

use std::path::Path;

use super::db::*;
use admin_server::AdminServerConfig;
use customer_server::CustomerServerConfig;
use lana_app::app::AppConfig;

/// Time configuration for the application clock
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TimeConfig {
    /// Use real system time
    #[default]
    Realtime,
    /// Use artificial/simulated time with configurable behavior
    Artificial(ArtificialClockConfig),
}

impl TimeConfig {
    pub(super) fn into_clock(self) -> (ClockHandle, Option<ClockController>) {
        match self {
            Self::Realtime => (ClockHandle::realtime(), None),
            Self::Artificial(cfg) => {
                let (clock, ctrl) = ClockHandle::artificial(cfg);
                (clock, Some(ctrl))
            }
        }
    }
}

/// Main configuration structure for the Lana banking application
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Database configuration for PostgreSQL connection
    #[serde(default)]
    pub db: DbConfig,
    /// Admin GraphQL API server configuration
    #[serde(default)]
    pub admin_server: AdminServerConfig,
    /// Customer-facing GraphQL API server configuration
    #[serde(default)]
    pub customer_server: CustomerServerConfig,
    /// Application-level configuration including jobs, KYC, custody, etc.
    #[serde(default)]
    pub app: AppConfig,
    /// OpenTelemetry tracing configuration for observability
    #[serde(default)]
    pub tracing: TracingConfig,
    /// Time configuration (realtime or artificial)
    #[serde(default)]
    pub time: TimeConfig,

    /// Bootstrap configuration for simulation setup (only available in sim-bootstrap feature)
    #[cfg(feature = "sim-bootstrap")]
    #[serde(default)]
    pub bootstrap: BootstrapConfig,
}

pub struct EnvSecrets {
    pub pg_con: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub encryption_key: String,
    pub deprecated_encryption_key: Option<String>,
    pub keycloak_internal_client_secret: String,
    pub keycloak_customer_client_secret: String,
}

impl Config {
    pub fn try_new(
        path: impl AsRef<Path>,
        EnvSecrets {
            pg_con,
            smtp_username,
            smtp_password,
            encryption_key,
            deprecated_encryption_key,
            keycloak_internal_client_secret,
            keycloak_customer_client_secret,
        }: EnvSecrets,
        config_overrides: &[String],
    ) -> anyhow::Result<Self> {
        let config_file = std::fs::read_to_string(&path)
            .context(format!("Couldn't read config file {:?}", path.as_ref()))?;

        let mut yaml_value: serde_yaml::Value =
            serde_yaml::from_str(&config_file).context("Couldn't parse config file")?;

        for override_str in config_overrides {
            let (key, value) = override_str.split_once('=').context(format!(
                "Invalid override format '{override_str}', expected KEY=VALUE"
            ))?;
            apply_yaml_override(&mut yaml_value, key, value)?;
        }

        let mut config: Config =
            serde_yaml::from_value(yaml_value).context("Couldn't deserialize config")?;

        config.db.pg_con.clone_from(&pg_con);
        config.app.notification.email.username = smtp_username;
        config.app.notification.email.password = smtp_password;
        config.app.user_onboarding.keycloak.client_secret = keycloak_internal_client_secret;
        config.app.customer_sync.keycloak.client_secret = keycloak_customer_client_secret;

        let parse_key = |hex_str: String| -> anyhow::Result<[u8; 32]> {
            let bytes = hex::decode(hex_str)?;
            bytes.try_into().map_err(|v: Vec<u8>| {
                anyhow::anyhow!("Encryption key must be 32 bytes, got {}", v.len())
            })
        };

        config.app.encryption.encryption_key =
            encryption::EncryptionKey::new(parse_key(encryption_key)?);
        config.app.encryption.deprecated_encryption_key = deprecated_encryption_key
            .map(parse_key)
            .transpose()?
            .map(encryption::EncryptionKey::new);

        Ok(config)
    }
}

/// Apply a dot-separated key override to a YAML value tree.
/// e.g. "bootstrap.seed_only" with value "true" sets yaml["bootstrap"]["seed_only"] = true
fn apply_yaml_override(root: &mut serde_yaml::Value, key: &str, value: &str) -> anyhow::Result<()> {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = root;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            let parsed_value = serde_yaml::from_str(value)
                .unwrap_or_else(|_| serde_yaml::Value::String(value.to_string()));
            current[*part] = parsed_value;
        } else {
            if !current.get(*part).is_some_and(|v| v.is_mapping()) {
                current[*part] = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
            }
            current = current
                .get_mut(*part)
                .context(format!("Failed to traverse config key '{key}'"))?;
        }
    }

    Ok(())
}
