use cala_ledger::CalaLedger;
use core_custody::{CustodianEncryptionConfig, CustodyConfig};

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON").unwrap();
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

pub fn custody_config() -> CustodyConfig {
    CustodyConfig {
        custodian_encryption: CustodianEncryptionConfig {
            key: [1u8; 32].into(),
        },
        deprecated_encryption_key: None,
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

pub mod action {
    use core_accounting::CoreAccountingAction;
    use core_credit::CoreCreditAction;
    use core_custody::CoreCustodyAction;
    use core_customer::CoreCustomerAction;
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

    use core_credit::CoreCreditEvent;
    use core_custody::CoreCustodyEvent;
    use core_customer::CoreCustomerEvent;
    use governance::GovernanceEvent;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        CoreCredit(CoreCreditEvent),
        CoreCustody(CoreCustodyEvent),
        CoreCustomer(CoreCustomerEvent),
        Governance(GovernanceEvent),
    }

    macro_rules! impl_event_marker {
        ($from_type:ty, $variant:ident) => {
            impl outbox::OutboxEventMarker<$from_type> for DummyEvent {
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

    impl_event_marker!(GovernanceEvent, Governance);
    impl_event_marker!(CoreCreditEvent, CoreCredit);
    impl_event_marker!(CoreCustodyEvent, CoreCustody);
    impl_event_marker!(CoreCustomerEvent, CoreCustomer);
}
