use serde::{Deserialize, Serialize};

use core_custody::CoreCustodyEvent;

pub(crate) async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON")?;
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

pub(crate) mod action {
    #[allow(dead_code)]
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(crate) struct DummyAction;

    impl From<core_custody::CoreCustodyAction> for DummyAction {
        fn from(_: core_custody::CoreCustodyAction) -> Self {
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

pub(crate) mod object {
    #[allow(dead_code)]
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(crate) struct DummyObject;

    impl From<core_custody::CoreCustodyObject> for DummyObject {
        fn from(_: core_custody::CoreCustodyObject) -> Self {
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

pub(crate) mod event {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, obix::OutboxEvent)]
    #[serde(tag = "module")]
    pub(crate) enum DummyEvent {
        CoreCustody(CoreCustodyEvent),
        #[serde(other)]
        Unknown,
    }

    pub(crate) use obix::test_utils::expect_event;
}
