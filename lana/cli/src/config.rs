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
    pub keycloak_internal_client_secret: String,
    pub keycloak_customer_client_secret: String,
}

impl Config {
    pub fn init(
        path: impl AsRef<Path>,
        EnvSecrets {
            pg_con,
            smtp_username,
            smtp_password,
            encryption_key,
            keycloak_internal_client_secret,
            keycloak_customer_client_secret,
        }: EnvSecrets,
    ) -> anyhow::Result<Self> {
        let config_file = std::fs::read_to_string(&path)
            .context(format!("Couldn't read config file {:?}", path.as_ref()))?;

        let mut config: Config =
            serde_yaml::from_str(&config_file).context("Couldn't parse config file")?;

        config.db.pg_con.clone_from(&pg_con);
        config.app.notification.email.username = smtp_username;
        config.app.notification.email.password = smtp_password;
        config.app.user_onboarding.keycloak.client_secret = keycloak_internal_client_secret;
        config.app.customer_sync.keycloak.client_secret = keycloak_customer_client_secret;

        let key_bytes = hex::decode(encryption_key)?;
        if key_bytes.len() != 32 {
            return Err(anyhow::anyhow!(
                "Custodian encryption key must be 32 bytes, got {}",
                key_bytes.len()
            ));
        }

        let key_array: [u8; 32] = key_bytes.as_slice().try_into().expect("key is 32 bytes");
        config.app.custody.encryption.key = key_array.into();

        Ok(config)
    }
}
