use anyhow::Context;
use lava_tracing::TracingConfig;
use serde::{Deserialize, Serialize};

use std::path::Path;

use super::db::*;
use crate::{app::AppConfig, server::admin::AdminServerConfig, server::public::PublicServerConfig};

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
    pub bfx_key: String,
    pub bfx_secret: String,
    pub sumsub_key: String,
    pub sumsub_secret: String,
}

impl Config {
    pub fn from_path(
        path: impl AsRef<Path>,
        EnvOverride {
            db_con,
            bfx_key,
            bfx_secret,
            sumsub_key,
            sumsub_secret,
        }: EnvOverride,
    ) -> anyhow::Result<Self> {
        let config_file = std::fs::read_to_string(&path)
            .context(format!("Couldn't read config file {:?}", path.as_ref()))?;
        let mut config: Config =
            serde_yaml::from_str(&config_file).context("Couldn't parse config file")?;
        config.db.pg_con.clone_from(&db_con);
        config.app.ledger.bfx_key = bfx_key;
        config.app.ledger.bfx_secret = bfx_secret;
        config.app.sumsub.sumsub_key = sumsub_key;
        config.app.sumsub.sumsub_secret = sumsub_secret;
        config.app.casbin.db_con = db_con;

        Ok(config)
    }
}
