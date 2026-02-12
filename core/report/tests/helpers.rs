use serde::{Deserialize, Serialize};

use core_report::CoreReportEvent;

pub(crate) async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON")?;
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

pub(crate) mod event {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, obix::OutboxEvent)]
    #[serde(tag = "module")]
    pub(crate) enum DummyEvent {
        CoreReport(CoreReportEvent),
        #[serde(other)]
        Unknown,
    }

    pub(crate) use obix::test_utils::expect_event;
}
