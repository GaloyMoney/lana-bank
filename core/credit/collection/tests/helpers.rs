#![allow(dead_code)]

use std::sync::Arc;

use cala_ledger::{CalaLedger, CalaLedgerConfig, account::NewAccount};
use core_credit_collection::{
    CalaAccountId, CollectionPublisher, CoreCreditCollection, ObligationReceivableAccountIds,
};
use es_entity::clock::ClockHandle;
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
    pub jobs: job::Jobs,
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
    let (clock, _ctrl) = ClockHandle::manual();

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
        .clock(clock.clone())
        .build()?;
    let cala = CalaLedger::init(cala_config).await?;
    let journal_id = init_journal(&cala).await?;

    // Payment ledger accounts used by `RecordPayment`/`RecordPaymentAllocation` templates.
    let payments_made_omnibus = create_account(&cala, "payments-made-omnibus").await?;
    let payment_source = create_account(&cala, "payment-source").await?;
    let payment_holding = create_account(&cala, "payment-holding").await?;
    let uncovered_outstanding = create_account(&cala, "uncovered-outstanding").await?;
    // Obligation lifecycle receivables used by due/overdue/defaulted postings.
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
            .clock(clock.clone())
            .build()
            .unwrap(),
    )
    .await?;

    let publisher = CollectionPublisher::new(&outbox);
    let collection_init = CoreCreditCollection::init(
        &pool,
        Arc::new(authz),
        &cala,
        journal_id,
        accounts.payments_made_omnibus,
        &mut jobs,
        &outbox,
        &publisher,
        clock.clone(),
    )
    .await?;
    let collections = collection_init.service;

    // Wire up EOD orchestration so EndOfDay events trigger the full pipeline
    let deposit_activity_spawner = jobs.add_initializer(noop_eod_job::NoopDepositActivityInit);
    let credit_facility_eod_spawner = jobs.add_initializer(noop_eod_job::NoopCreditFacilityEodInit);
    let _core_eod = core_eod::CoreEod::init(
        &pool,
        &mut jobs,
        &outbox,
        &outbox,
        clock.clone(),
        collection_init.obligation_status_spawner,
        deposit_activity_spawner,
        credit_facility_eod_spawner,
    )
    .await?;

    Ok(TestContext {
        pool,
        clock,
        outbox,
        collections,
        jobs,
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
    use core_eod::CoreEodEvent;
    use core_time_events::CoreTimeEvent;

    #[derive(Debug, Serialize, Deserialize, obix::OutboxEvent)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        CoreCreditCollection(CoreCreditCollectionEvent),
        CoreTimeEvent(CoreTimeEvent),
        CoreEod(CoreEodEvent),
        #[serde(other)]
        Unknown,
    }

    pub use obix::test_utils::expect_event;
}

/// Noop job initializers for EOD child processes that are not needed in
/// these tests but must exist so the EodProcessManager can advance.
mod noop_eod_job {
    use async_trait::async_trait;
    use job::*;

    pub struct NoopDepositActivityInit;

    impl JobInitializer for NoopDepositActivityInit {
        type Config = core_eod::deposit_activity_process::DepositActivityProcessConfig;

        fn job_type(&self) -> JobType {
            core_eod::deposit_activity_process::DEPOSIT_ACTIVITY_PROCESS_JOB
        }

        fn init(
            &self,
            _job: &Job,
            _spawner: JobSpawner<Self::Config>,
        ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
            Ok(Box::new(NoopRunner))
        }
    }

    pub struct NoopCreditFacilityEodInit;

    impl JobInitializer for NoopCreditFacilityEodInit {
        type Config = core_eod::credit_facility_eod_process::CreditFacilityEodProcessConfig;

        fn job_type(&self) -> JobType {
            core_eod::credit_facility_eod_process::CREDIT_FACILITY_EOD_PROCESS_JOB
        }

        fn init(
            &self,
            _job: &Job,
            _spawner: JobSpawner<Self::Config>,
        ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
            Ok(Box::new(NoopRunner))
        }
    }

    struct NoopRunner;

    #[async_trait]
    impl JobRunner for NoopRunner {
        async fn run(
            &self,
            _current_job: CurrentJob,
        ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
            Ok(JobCompletion::Complete)
        }
    }
}
