use serde::{Deserialize, Serialize};

use core_report::CoreReportEvent;

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON")?;
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

pub mod event {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        CoreReport(CoreReportEvent),
        #[serde(other)]
        Unknown,
    }

    impl obix::out::OutboxEventMarker<CoreReportEvent> for DummyEvent {
        fn as_event(&self) -> Option<&CoreReportEvent> {
            match self {
                Self::CoreReport(event) => Some(event),
                Self::Unknown => None,
            }
        }
    }

    impl From<CoreReportEvent> for DummyEvent {
        fn from(event: CoreReportEvent) -> Self {
            Self::CoreReport(event)
        }
    }

    pub use obix::test_utils::expect_event;
}
