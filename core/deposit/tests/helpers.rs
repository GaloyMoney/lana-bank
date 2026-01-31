#![allow(dead_code)] // Helper functions may not be used in all tests

use cala_ledger::CalaLedger;
use core_accounting::{AccountingBaseConfig, CoreAccounting};
use domain_config::{
    ExposedDomainConfigs, ExposedDomainConfigsReadOnly, InternalDomainConfigs,
    RequireVerifiedCustomerForAccount,
};
use rand::Rng;

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON").unwrap();
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

pub async fn init_read_only_exposed_domain_configs(
    pool: &sqlx::PgPool,
    authz: &authz::dummy::DummyPerms<action::DummyAction, object::DummyObject>,
) -> anyhow::Result<ExposedDomainConfigsReadOnly> {
    let exposed_configs = ExposedDomainConfigs::new(pool, authz);
    exposed_configs.seed_registered().await?;
    // Disable the require verified customer check for tests
    // Ignore concurrent modification - all tests want the same value (false)
    let _ = exposed_configs
        .update::<RequireVerifiedCustomerForAccount>(&authz::dummy::DummySubject, false)
        .await;
    Ok(ExposedDomainConfigsReadOnly::new(pool))
}

pub async fn init_internal_domain_configs(
    pool: &sqlx::PgPool,
) -> anyhow::Result<InternalDomainConfigs> {
    clear_internal_domain_config(pool, "deposit-chart-of-accounts-integration").await?;
    let internal_configs = InternalDomainConfigs::new(pool);
    internal_configs.seed_registered().await?;
    Ok(internal_configs)
}

async fn clear_internal_domain_config(pool: &sqlx::PgPool, key: &str) -> anyhow::Result<()> {
    // Use a CTE to perform all deletes atomically in dependency order.
    // This prevents race conditions when tests run in parallel.
    sqlx::query(
        "WITH config_ids AS (
            SELECT id FROM core_domain_configs WHERE key = $1 FOR UPDATE
        ),
        deleted_rollup AS (
            DELETE FROM core_domain_config_events_rollup
            WHERE id IN (SELECT id FROM config_ids)
        ),
        deleted_events AS (
            DELETE FROM core_domain_config_events
            WHERE id IN (SELECT id FROM config_ids)
        )
        DELETE FROM core_domain_configs WHERE id IN (SELECT id FROM config_ids)",
    )
    .bind(key)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn init_journal(cala: &CalaLedger) -> anyhow::Result<cala_ledger::JournalId> {
    use cala_ledger::journal::*;

    let id = JournalId::new();
    let new = NewJournal::builder()
        .id(id)
        .name("Test journal")
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
        "4".parse().unwrap(),
        "5".parse().unwrap(),
        "6".parse().unwrap(),
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
    use core_customer::CoreCustomerAction;
    use core_deposit::{CoreDepositAction, GovernanceAction};
    use domain_config::DomainConfigAction;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyAction;

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

    impl From<CoreCustomerAction> for DummyAction {
        fn from(_: CoreCustomerAction) -> Self {
            Self
        }
    }

    impl From<CoreAccountingAction> for DummyAction {
        fn from(_: CoreAccountingAction) -> Self {
            Self
        }
    }

    impl From<DomainConfigAction> for DummyAction {
        fn from(_: DomainConfigAction) -> Self {
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
    use core_customer::CustomerObject;
    use core_deposit::{CoreDepositObject, GovernanceObject};
    use domain_config::DomainConfigObject;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyObject;

    impl From<CoreDepositObject> for DummyObject {
        fn from(_: CoreDepositObject) -> Self {
            Self
        }
    }
    impl From<CoreAccountingObject> for DummyObject {
        fn from(_: CoreAccountingObject) -> Self {
            Self
        }
    }

    impl From<GovernanceObject> for DummyObject {
        fn from(_: GovernanceObject) -> Self {
            Self
        }
    }

    impl From<CustomerObject> for DummyObject {
        fn from(_: CustomerObject) -> Self {
            Self
        }
    }

    impl From<DomainConfigObject> for DummyObject {
        fn from(_: DomainConfigObject) -> Self {
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
    use serde::{Deserialize, Serialize};

    use core_accounting::CoreAccountingEvent;
    use core_customer::CoreCustomerEvent;
    use core_deposit::CoreDepositEvent;
    use governance::GovernanceEvent;

    #[derive(Debug, Serialize, Deserialize, obix::OutboxEvent)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        CoreDeposit(CoreDepositEvent),
        CoreCustomer(CoreCustomerEvent),
        CoreAccounting(CoreAccountingEvent),
        Governance(GovernanceEvent),
        #[serde(other)]
        Unknown,
    }
}
