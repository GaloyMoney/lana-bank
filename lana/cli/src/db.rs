use serde::{Deserialize, Serialize};

/// Database connection configuration for PostgreSQL
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DbConfig {
    /// PostgreSQL connection string (provided via PG_CON env var)
    #[serde(skip)]
    pub pg_con: String,
    /// Maximum number of connections in the connection pool
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
}

pub async fn init_pool(config: &DbConfig) -> anyhow::Result<sqlx::PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.pool_size)
        .connect(&config.pg_con)
        .await?;

    Ok(pool)
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            pg_con: "".to_string(),
            pool_size: default_pool_size(),
        }
    }
}

fn default_pool_size() -> u32 {
    20
}
