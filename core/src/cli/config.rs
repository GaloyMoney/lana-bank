use anyhow::Context;
use lava_tracing::TracingConfig;
use serde::{Deserialize, Serialize};

use std::path::Path;

use super::db::*;
use crate::{
    app::AppConfig,
    server::{admin::AdminServerConfig, public::PublicServerConfig},
};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub db: DbConfig,
    #[serde(default)]
    pub public_server: PublicServerConfig,
    #[serde(default)]
    pub admin_server: AdminServerConfig,
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub tracing: TracingConfig,
}

pub struct EnvOverride {
    pub db_con: String,
    pub sumsub_key: String,
    pub sumsub_secret: String,
    pub sa_creds_base64: String,
}

impl Config {
    pub fn from_path(
        path: impl AsRef<Path>,
        EnvOverride {
            db_con,
            sumsub_key,
            sumsub_secret,
            sa_creds_base64,
        }: EnvOverride,
    ) -> anyhow::Result<Self> {
        let config_file = std::fs::read_to_string(&path)
            .context(format!("Couldn't read config file {:?}", path.as_ref()))?;
        let mut config: Config =
            serde_yaml::from_str(&config_file).context("Couldn't parse config file")?;
        config.db.pg_con.clone_from(&db_con);
        config.app.sumsub.sumsub_key = sumsub_key;
        config.app.sumsub.sumsub_secret = sumsub_secret;
        config.app.report.set_sa_creds_base64(sa_creds_base64)?;

        Ok(config)
    }
}
