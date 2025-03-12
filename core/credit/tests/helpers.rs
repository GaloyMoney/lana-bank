use cala_ledger::CalaLedger;

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_host = std::env::var("PG_HOST").unwrap_or("localhost".to_string());
    let pg_con = format!("postgres://user:password@{pg_host}:5433/pg");
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
    use chart_of_accounts::CoreChartOfAccountsAction;
    use core_credit::CoreCreditAction;
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

    impl From<CoreChartOfAccountsAction> for DummyAction {
        fn from(_: CoreChartOfAccountsAction) -> Self {
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
    use chart_of_accounts::CoreChartOfAccountsObject;
    use core_credit::CoreCreditObject;
    use core_customer::CustomerObject;
    use governance::GovernanceObject;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyObject;

    impl From<CoreCreditObject> for DummyObject {
        fn from(_: CoreCreditObject) -> Self {
            Self
        }
    }
    impl From<CoreChartOfAccountsObject> for DummyObject {
        fn from(_: CoreChartOfAccountsObject) -> Self {
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

    use core_credit::CoreCreditEvent;
    use core_customer::CoreCustomerEvent;
    use governance::GovernanceEvent;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        CoreCredit(CoreCreditEvent),
        CoreCustomer(CoreCustomerEvent),
        Governance(GovernanceEvent),
    }

    macro_rules! impl_event_marker {
        ($from_type:ty, $variant:ident) => {
            impl outbox::OutboxEventMarker<$from_type> for DummyEvent {
                fn as_event(&self) -> Option<&$from_type> {
                    match self {
                        Self::$variant(ref event) => Some(event),
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
    impl_event_marker!(CoreCustomerEvent, CoreCustomer);
}
