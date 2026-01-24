#![allow(dead_code)] // Helper functions may not be used in all tests

use cala_ledger::CalaLedger;
use core_accounting::AccountingBaseConfig;
use core_custody::{CustodyConfig, EncryptionConfig};
use domain_config::{ExposedDomainConfigs, ExposedDomainConfigsReadOnly};

use core_credit::CustomerActiveCheckEnabled;
use core_deposit::RequireVerifiedCustomerForAccount;

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON").unwrap();
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

pub async fn init_domain_configs(
    pool: &sqlx::PgPool,
    authz: &authz::dummy::DummyPerms<action::DummyAction, object::DummyObject>,
) -> anyhow::Result<ExposedDomainConfigsReadOnly> {
    let exposed_configs = ExposedDomainConfigs::new(pool, authz);
    exposed_configs.seed_registered().await?;
    // Disable the customer active check for tests
    exposed_configs
        .update::<CustomerActiveCheckEnabled>(&authz::dummy::DummySubject, false)
        .await?;
    // Disable the require verified customer check for deposit tests
    exposed_configs
        .update::<RequireVerifiedCustomerForAccount>(&authz::dummy::DummySubject, false)
        .await?;
    Ok(ExposedDomainConfigsReadOnly::new(pool))
}

pub fn custody_config() -> CustodyConfig {
    CustodyConfig {
        encryption: EncryptionConfig {
            key: [1u8; 32].into(),
        },
        deprecated_encryption_key: None,
        custody_providers: Default::default(),
    }
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
8,,,Off Balance Sheet,Credit,
"#;

pub mod action {
    use core_accounting::CoreAccountingAction;
    use core_credit::CoreCreditAction;
    use core_custody::CoreCustodyAction;
    use core_customer::CoreCustomerAction;
    use core_deposit::CoreDepositAction;
    use domain_config::DomainConfigAction;
    use governance::GovernanceAction;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyAction;

    impl From<CoreCreditAction> for DummyAction {
        fn from(_: CoreCreditAction) -> Self {
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

    impl From<CoreCustodyAction> for DummyAction {
        fn from(_: CoreCustodyAction) -> Self {
            Self
        }
    }

    impl From<CoreAccountingAction> for DummyAction {
        fn from(_: CoreAccountingAction) -> Self {
            Self
        }
    }

    impl From<CoreDepositAction> for DummyAction {
        fn from(_: CoreDepositAction) -> Self {
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
    use core_credit::CoreCreditObject;
    use core_custody::CoreCustodyObject;
    use core_customer::CustomerObject;
    use core_deposit::CoreDepositObject;
    use domain_config::DomainConfigObject;
    use governance::GovernanceObject;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyObject;

    impl From<CoreCreditObject> for DummyObject {
        fn from(_: CoreCreditObject) -> Self {
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

    impl From<CoreCustodyObject> for DummyObject {
        fn from(_: CoreCustodyObject) -> Self {
            Self
        }
    }

    impl From<CoreDepositObject> for DummyObject {
        fn from(_: CoreDepositObject) -> Self {
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

    use core_access::CoreAccessEvent;
    use core_accounting::CoreAccountingEvent;
    use core_credit::CoreCreditEvent;
    use core_custody::CoreCustodyEvent;
    use core_customer::CoreCustomerEvent;
    use core_deposit::CoreDepositEvent;
    use core_price::CorePriceEvent;
    use governance::GovernanceEvent;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        CoreAccess(CoreAccessEvent),
        CoreAccounting(CoreAccountingEvent),
        CoreCredit(CoreCreditEvent),
        CoreCustody(CoreCustodyEvent),
        CoreCustomer(CoreCustomerEvent),
        CoreDeposit(CoreDepositEvent),
        Governance(GovernanceEvent),
        Price(CorePriceEvent),
    }

    macro_rules! impl_event_marker {
        ($from_type:ty, $variant:ident) => {
            impl obix::out::OutboxEventMarker<$from_type> for DummyEvent {
                fn as_event(&self) -> Option<&$from_type> {
                    match self {
                        &Self::$variant(ref event) => Some(event),
                        _ => None,
                    }
                }
            }
            impl From<$from_type> for DummyEvent {
                fn from(event: $from_type) -> Self {
                    Self::$variant(event)
                }
            }
        };
    }

    impl_event_marker!(CoreAccessEvent, CoreAccess);
    impl_event_marker!(CoreAccountingEvent, CoreAccounting);
    impl_event_marker!(GovernanceEvent, Governance);
    impl_event_marker!(CoreCreditEvent, CoreCredit);
    impl_event_marker!(CoreCustodyEvent, CoreCustody);
    impl_event_marker!(CoreCustomerEvent, CoreCustomer);
    impl_event_marker!(CoreDepositEvent, CoreDeposit);
    impl_event_marker!(CorePriceEvent, Price);
}
