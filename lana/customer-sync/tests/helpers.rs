#![allow(dead_code)]

use cloud_storage::{Storage, config::StorageConfig};
use core_customer::Customers;
use document_storage::DocumentStorage;
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use obix::test_utils::expect_event;
use public_id::PublicIds;

pub mod action {
    use core_customer::CoreCustomerAction;
    use core_deposit::CoreDepositAction;
    use governance::GovernanceAction;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyAction;

    impl From<CoreCustomerAction> for DummyAction {
        fn from(_: CoreCustomerAction) -> Self {
            Self
        }
    }

    impl From<CoreDepositAction> for DummyAction {
        fn from(_: CoreDepositAction) -> Self {
            Self
        }
    }

    impl From<GovernanceAction> for DummyAction {
        fn from(_: GovernanceAction) -> Self {
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
    use core_customer::CustomerObject;
    use core_deposit::CoreDepositObject;
    use governance::GovernanceObject;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyObject;

    impl From<CustomerObject> for DummyObject {
        fn from(_: CustomerObject) -> Self {
            Self
        }
    }

    impl From<CoreDepositObject> for DummyObject {
        fn from(_: CoreDepositObject) -> Self {
            Self
        }
    }

    impl From<GovernanceObject> for DummyObject {
        fn from(_: GovernanceObject) -> Self {
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
    use serde::{Deserialize, Serialize};

    use core_customer::CoreCustomerEvent;
    use core_deposit::CoreDepositEvent;
    use core_time_events::CoreTimeEvent;
    use governance::GovernanceEvent;
    use lana_events::LanaEvent;

    #[derive(Debug, Serialize, Deserialize, obix::OutboxEvent)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        CoreCustomer(CoreCustomerEvent),
        CoreDeposit(CoreDepositEvent),
        Governance(GovernanceEvent),
        CoreTimeEvent(CoreTimeEvent),
        Lana(LanaEvent),
        #[serde(other)]
        Unknown,
    }
}

pub type TestPerms = authz::dummy::DummyPerms<action::DummyAction, object::DummyObject>;
pub type TestEvent = event::DummyEvent;

pub struct TestContext {
    pub customers: Customers<TestPerms, TestEvent>,
    pub customer_activity_repo: core_customer::CustomerActivityRepo,
    pub outbox: obix::Outbox<TestEvent>,
    pub jobs: job::Jobs,
}

pub async fn expect_handler_reaction<IE, T, M>(
    outbox: &obix::Outbox<TestEvent>,
    input_event: impl Into<TestEvent>,
    matches: M,
) -> anyhow::Result<T>
where
    TestEvent: obix::out::OutboxEventMarker<IE>,
    IE: Send + Sync + 'static,
    M: Fn(&(), &IE) -> Option<T>,
{
    let outbox_clone = outbox.clone();
    let event = input_event.into();
    let (_, extracted) = expect_event(
        outbox,
        || async {
            let mut op = outbox_clone.begin_op().await?;
            outbox_clone.publish_persisted_in_op(&mut op, event).await?;
            op.commit().await?;
            Ok::<_, sqlx::Error>(())
        },
        matches,
    )
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(extracted)
}

pub async fn setup() -> anyhow::Result<TestContext> {
    let pg_con = std::env::var("PG_CON").unwrap();
    let pool = sqlx::PgPool::connect(&pg_con).await?;

    // Remove stale outbox handler jobs from previous test runs so that
    // register_event_handler / spawn_unique creates fresh ones.
    // Pattern from: obix-0.2.14/tests/helpers.rs::wipeout_outbox_job_tables
    let job_type = "outbox.update-customer-activity-status";
    sqlx::query("DELETE FROM job_events WHERE id IN (SELECT id FROM jobs WHERE job_type = $1)")
        .bind(job_type)
        .execute(&pool)
        .await?;
    sqlx::query("DELETE FROM job_executions WHERE job_type = $1")
        .bind(job_type)
        .execute(&pool)
        .await?;
    sqlx::query("DELETE FROM jobs WHERE job_type = $1")
        .bind(job_type)
        .execute(&pool)
        .await?;

    let (clock, _ctrl) = ClockHandle::artificial(ArtificialClockConfig::manual());

    let outbox =
        obix::Outbox::<TestEvent>::init(&pool, obix::MailboxConfig::builder().build()?).await?;

    let authz = TestPerms::new();
    let storage = Storage::new(&StorageConfig::default());
    let document_storage = DocumentStorage::new(&pool, &storage, clock.clone());
    let public_ids = PublicIds::new(&pool);
    let customers = Customers::new(
        &pool,
        &authz,
        &outbox,
        document_storage,
        public_ids,
        clock.clone(),
    );

    let mut jobs = job::Jobs::init(
        job::JobSvcConfig::builder()
            .pool(pool.clone())
            .build()
            .unwrap(),
    )
    .await?;

    outbox
        .register_event_handler(
            &mut jobs,
            obix::out::OutboxEventJobConfig::new(
                customer_sync::jobs::UPDATE_CUSTOMER_ACTIVITY_STATUS,
            ),
            customer_sync::jobs::UpdateCustomerActivityStatusHandler::new(&customers),
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let customer_activity_repo = core_customer::CustomerActivityRepo::new(pool.clone());

    Ok(TestContext {
        customers,
        customer_activity_repo,
        outbox,
        jobs,
    })
}
