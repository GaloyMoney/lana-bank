use serde::{Deserialize, Serialize};

use core_customer::CoreCustomerEvent;

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON")?;
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

pub mod action {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyAction;

    impl From<core_customer::CoreCustomerAction> for DummyAction {
        fn from(_: core_customer::CoreCustomerAction) -> Self {
            Self
        }
    }

    impl std::fmt::Display for DummyAction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "dummy")
        }
    }

    impl std::str::FromStr for DummyAction {
        type Err = strum::ParseError;

        fn from_str(_: &str) -> Result<Self, Self::Err> {
            Ok(Self)
        }
    }
}

pub mod object {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyObject;

    impl From<core_customer::CustomerObject> for DummyObject {
        fn from(_: core_customer::CustomerObject) -> Self {
            Self
        }
    }

    impl std::fmt::Display for DummyObject {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Dummy")
        }
    }

    impl std::str::FromStr for DummyObject {
        type Err = &'static str;

        fn from_str(_: &str) -> Result<Self, Self::Err> {
            Ok(DummyObject)
        }
    }
}

pub mod event {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        CoreCustomer(CoreCustomerEvent),
    }

    impl obix::out::OutboxEventMarker<CoreCustomerEvent> for DummyEvent {
        fn as_event(&self) -> Option<&CoreCustomerEvent> {
            match self {
                Self::CoreCustomer(event) => Some(event),
            }
        }
    }

    impl From<CoreCustomerEvent> for DummyEvent {
        fn from(event: CoreCustomerEvent) -> Self {
            Self::CoreCustomer(event)
        }
    }

    pub use obix::test_utils::expect_event;
}
