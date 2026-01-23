use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_stream::StreamExt;

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

    pub async fn expect_event<R, T, F, Fut, E, P>(
        outbox: &obix::Outbox<DummyEvent>,
        use_case: F,
        matches: P,
    ) -> anyhow::Result<(R, T)>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<R, E>>,
        E: std::fmt::Debug,
        P: Fn(&R, &CoreCustomerEvent) -> Option<T>,
    {
        let mut listener = outbox.listen_persisted(None);

        let result = use_case().await.expect("use case should succeed");

        let event = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let event = listener.next().await.expect("should receive an event");
                if let Some(extracted) = event
                    .as_event::<CoreCustomerEvent>()
                    .and_then(|e| matches(&result, e))
                {
                    return extracted;
                }
            }
        })
        .await?;

        Ok((result, event))
    }
}
