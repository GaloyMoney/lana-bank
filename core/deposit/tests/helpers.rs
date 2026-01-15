use cala_ledger::CalaLedger;

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
        .build()
        .unwrap();
    let journal = cala.journals().create(new).await?;
    Ok(journal.id)
}

pub mod action {
    use core_accounting::CoreAccountingAction;
    use core_customer::CoreCustomerAction;
    use core_deposit::{CoreDepositAction, GovernanceAction};

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

    use core_customer::CoreCustomerEvent;
    use core_deposit::CoreDepositEvent;
    use document_storage::CoreDocumentStorageEvent;
    use governance::GovernanceEvent;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        CoreDeposit(CoreDepositEvent),
        CoreCustomer(CoreCustomerEvent),
        DocumentStorage(CoreDocumentStorageEvent),
        Governance(GovernanceEvent),
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

    impl_event_marker!(GovernanceEvent, Governance);
    impl_event_marker!(CoreDepositEvent, CoreDeposit);
    impl_event_marker!(CoreCustomerEvent, CoreCustomer);
    impl_event_marker!(CoreDocumentStorageEvent, DocumentStorage);
}
