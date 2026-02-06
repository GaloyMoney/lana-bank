#![allow(dead_code)]

use std::sync::Arc;

use cala_ledger::{CalaLedger, CalaLedgerConfig, account::NewAccount};
use core_credit_collection::{
    CalaAccountId, CollectionPublisher, CoreCreditCollection, ObligationReceivableAccountIds,
};
use es_entity::clock::{ArtificialClockConfig, ClockHandle};
use obix::Outbox;

use event::DummyEvent;

pub type TestPerms = authz::dummy::DummyPerms<action::DummyAction, object::DummyObject>;

pub struct TestAccounts {
    pub receivable: ObligationReceivableAccountIds,
    pub defaulted: CalaAccountId,
    pub payment_source: CalaAccountId,
    pub payment_holding: CalaAccountId,
    pub uncovered_outstanding: CalaAccountId,
    pub payments_made_omnibus: CalaAccountId,
}

pub struct TestContext {
    pub pool: sqlx::PgPool,
    pub clock: ClockHandle,
    pub outbox: Outbox<DummyEvent>,
    pub collections: CoreCreditCollection<TestPerms, DummyEvent>,
    pub _jobs: job::Jobs,
    pub accounts: TestAccounts,
}

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON").unwrap();
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

pub async fn init_journal(cala: &CalaLedger) -> anyhow::Result<cala_ledger::JournalId> {
    use cala_ledger::journal::*;

    let id = JournalId::new();
    let new = NewJournal::builder()
        .id(id)
        .name("Test journal")
        .enable_effective_balance(true)
        .build()
        .unwrap();
    let journal = cala.journals().create(new).await?;
    Ok(journal.id)
}

pub async fn create_account(cala: &CalaLedger, prefix: &str) -> anyhow::Result<CalaAccountId> {
    let id = CalaAccountId::new();
    let new_account = NewAccount::builder()
        .id(id)
        .code(id.to_string())
        .name(prefix)
        .build()
        .expect("could not build ledger account");
    let account = cala.accounts().create(new_account).await?;
    Ok(account.id)
}

pub async fn setup() -> anyhow::Result<TestContext> {
    let pool = init_pool().await?;
    let (clock, _ctrl) = ClockHandle::artificial(ArtificialClockConfig::manual());

    let outbox = Outbox::<DummyEvent>::init(
        &pool,
        obix::MailboxConfig::builder()
            .clock(clock.clone())
            .build()?,
    )
    .await?;

    let authz = TestPerms::new();

    let cala_config = CalaLedgerConfig::builder()
        .pool(pool.clone())
        .exec_migrations(false)
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let journal_id = init_journal(&cala).await?;

    let payments_made_omnibus = create_account(&cala, "payments-made-omnibus").await?;
    let payment_source = create_account(&cala, "payment-source").await?;
    let payment_holding = create_account(&cala, "payment-holding").await?;
    let uncovered_outstanding = create_account(&cala, "uncovered-outstanding").await?;
    let receivable_not_yet_due = create_account(&cala, "receivable-not-yet-due").await?;
    let receivable_due = create_account(&cala, "receivable-due").await?;
    let receivable_overdue = create_account(&cala, "receivable-overdue").await?;
    let defaulted = create_account(&cala, "receivable-defaulted").await?;

    let accounts = TestAccounts {
        receivable: ObligationReceivableAccountIds {
            not_yet_due: receivable_not_yet_due,
            due: receivable_due,
            overdue: receivable_overdue,
        },
        defaulted,
        payment_source,
        payment_holding,
        uncovered_outstanding,
        payments_made_omnibus,
    };

    let mut jobs = job::Jobs::init(
        job::JobSvcConfig::builder()
            .pool(pool.clone())
            .build()
            .unwrap(),
    )
    .await?;

    let publisher = CollectionPublisher::new(&outbox);
    let collections = CoreCreditCollection::init(
        &pool,
        Arc::new(authz),
        &cala,
        journal_id,
        accounts.payments_made_omnibus,
        &mut jobs,
        &publisher,
        clock.clone(),
    )
    .await?;

    Ok(TestContext {
        pool,
        clock,
        outbox,
        collections,
        _jobs: jobs,
        accounts,
    })
}

pub mod action {
    use core_credit_collection::CoreCreditCollectionAction;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyAction;

    impl From<CoreCreditCollectionAction> for DummyAction {
        fn from(_: CoreCreditCollectionAction) -> Self {
            Self
        }
    }

    impl std::fmt::Display for DummyAction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "dummy")?;
            Ok(())
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
    use core_credit_collection::CoreCreditCollectionObject;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyObject;

    impl From<CoreCreditCollectionObject> for DummyObject {
        fn from(_: CoreCreditCollectionObject) -> Self {
            Self
        }
    }

    impl std::fmt::Display for DummyObject {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "dummy")?;
            Ok(())
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

    use core_credit_collection::CoreCreditCollectionEvent;

    #[derive(Debug, Serialize, Deserialize, obix::OutboxEvent)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        CoreCreditCollection(CoreCreditCollectionEvent),
        #[serde(other)]
        Unknown,
    }

    pub use obix::test_utils::expect_event;
}
