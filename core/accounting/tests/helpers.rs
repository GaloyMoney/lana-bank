use cala_ledger::CalaLedger;
use obix::out::Outbox;
use rand::Rng;

use core_accounting::{AccountingBaseConfig, CoreAccounting};

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON").unwrap();
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

pub async fn init_outbox(pool: &sqlx::PgPool) -> anyhow::Result<Outbox<event::TestEvent>> {
    let outbox = Outbox::init(pool, obix::MailboxConfig::builder().build()?).await?;
    Ok(outbox)
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

pub fn default_accounting_base_config() -> AccountingBaseConfig {
    AccountingBaseConfig::try_new(
        "1".parse().unwrap(),
        "2".parse().unwrap(),
        "3".parse().unwrap(),
        "32.01".parse().unwrap(),
        "32.02".parse().unwrap(),
        "7".parse().unwrap(),
        "8".parse().unwrap(),
        "4".parse().unwrap(),
        "5".parse().unwrap(),
        "6".parse().unwrap(),
        "9".parse().unwrap(),
    )
    .unwrap()
}

pub const BASE_ACCOUNTS_CSV: &str = r#"
1,,,Assets,Debit,
2,,,Liabilities,Credit,
3,,,Equity,Credit,
32,,,Retained Earnings,,
,01,,Annual Gains,,
,02,,Annual Losses,,
4,,,Revenue,Credit,
5,,,Cost of Revenue,Debit,
6,,,Expenses,Debit,
7,,,Contingent Rights,Debit,
8,,,Contingent Obligations,Credit,
9,,,Memorandum,Debit,
"#;

pub async fn create_test_statements<Perms, E>(
    accounting: &CoreAccounting<Perms, E>,
) -> anyhow::Result<(String, String, String)>
where
    Perms: authz::PermissionCheck,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Action:
        From<core_accounting::CoreAccountingAction>,
    <<Perms as authz::PermissionCheck>::Audit as audit::AuditSvc>::Object:
        From<core_accounting::CoreAccountingObject>,
    E: obix::out::OutboxEventMarker<core_accounting::CoreAccountingEvent>,
{
    let bs = format!("BS-{:08}", rand::rng().random_range(0..10000));
    let pl = format!("PL-{:08}", rand::rng().random_range(0..10000));
    let tb = format!("TB-{:08}", rand::rng().random_range(0..10000));

    accounting
        .balance_sheets()
        .create_balance_sheet(bs.clone())
        .await?;
    accounting
        .profit_and_loss()
        .create_pl_statement(pl.clone())
        .await?;
    accounting
        .trial_balances()
        .create_trial_balance_statement(tb.clone())
        .await?;

    Ok((bs, pl, tb))
}

pub mod action {
    use core_accounting::CoreAccountingAction;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyAction;

    impl From<CoreAccountingAction> for DummyAction {
        fn from(_: CoreAccountingAction) -> Self {
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
    use core_accounting::CoreAccountingObject;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyObject;

    impl From<CoreAccountingObject> for DummyObject {
        fn from(_: CoreAccountingObject) -> Self {
            Self
        }
    }

    impl std::fmt::Display for DummyObject {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Dummy")?;
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
    use core_accounting::CoreAccountingEvent;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, obix::OutboxEvent)]
    #[serde(tag = "module")]
    pub enum TestEvent {
        Accounting(CoreAccountingEvent),
        #[serde(other)]
        Unknown,
    }
}
