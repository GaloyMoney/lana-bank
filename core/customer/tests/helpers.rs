use cloud_storage::{Storage, config::StorageConfig};
use document_storage::DocumentStorage;
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use serde::{Deserialize, Serialize};

use core_customer::{CoreCustomerEvent, Customers};

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON")?;
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

/// Creates a test setup with all required dependencies for customer tests.
pub async fn setup() -> anyhow::Result<(
    Customers<
        authz::dummy::DummyPerms<action::DummyAction, object::DummyObject>,
        event::DummyEvent,
    >,
    obix::Outbox<event::DummyEvent>,
)> {
    let pool = init_pool().await?;
    let (clock, _time) = ClockHandle::artificial(ArtificialClockConfig::manual());

    let outbox = obix::Outbox::<event::DummyEvent>::init(
        &pool,
        obix::MailboxConfig::builder()
            .clock(clock.clone())
            .build()?,
    )
    .await?;

    let authz = authz::dummy::DummyPerms::<action::DummyAction, object::DummyObject>::new();
    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage, clock.clone());
    let public_ids = public_id::PublicIds::new(&pool);

    let customers = Customers::new(
        &pool,
        &authz,
        &outbox,
        document_storage,
        public_ids,
        clock.clone(),
    );

    Ok((customers, outbox))
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

    #[derive(Debug, Serialize, Deserialize, obix::OutboxEvent)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        CoreCustomer(CoreCustomerEvent),
        #[serde(other)]
        Unknown,
    }

    pub use obix::test_utils::expect_event;
}
