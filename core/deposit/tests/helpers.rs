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
    use deposit::{CoreDepositAction, GovernanceAction};

    #[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
    #[strum_discriminants(derive(strum::Display, strum::EnumString))]
    #[strum_discriminants(strum(serialize_all = "kebab-case"))]
    pub enum DummyAction {
        CoreDeposit(CoreDepositAction),
        Governance(GovernanceAction),
    }

    impl From<CoreDepositAction> for DummyAction {
        fn from(action: CoreDepositAction) -> Self {
            DummyAction::CoreDeposit(action)
        }
    }

    impl From<GovernanceAction> for DummyAction {
        fn from(action: GovernanceAction) -> Self {
            DummyAction::Governance(action)
        }
    }

    impl std::fmt::Display for DummyAction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}:", DummyActionDiscriminants::from(self))?;
            use DummyAction::*;
            match self {
                CoreDeposit(action) => action.fmt(f),
                Governance(action) => action.fmt(f),
            }
        }
    }

    impl std::str::FromStr for DummyAction {
        type Err = strum::ParseError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let (module, action) = s.split_once(':').expect("missing colon");
            use DummyActionDiscriminants::*;
            let res = match module.parse()? {
                CoreDeposit => DummyAction::from(action.parse::<CoreDepositAction>()?),
                Governance => DummyAction::from(action.parse::<GovernanceAction>()?),
            };
            Ok(res)
        }
    }
}

pub mod object {
    use deposit::{CoreDepositObject, GovernanceObject};

    #[derive(Clone, Copy, Debug, PartialEq, strum::EnumDiscriminants)]
    #[strum_discriminants(derive(strum::Display, strum::EnumString))]
    #[strum_discriminants(strum(serialize_all = "kebab-case"))]
    pub enum DummyObject {
        CoreDeposit(CoreDepositObject),
        Governance(GovernanceObject),
    }

    impl From<CoreDepositObject> for DummyObject {
        fn from(action: CoreDepositObject) -> Self {
            DummyObject::CoreDeposit(action)
        }
    }

    impl From<GovernanceObject> for DummyObject {
        fn from(action: GovernanceObject) -> Self {
            DummyObject::Governance(action)
        }
    }

    impl std::fmt::Display for DummyObject {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}/", DummyObjectDiscriminants::from(self))?;
            use DummyObject::*;
            match self {
                CoreDeposit(action) => action.fmt(f),
                Governance(action) => action.fmt(f),
            }
        }
    }

    impl std::str::FromStr for DummyObject {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let (module, object) = s.split_once('/').expect("missing colon");
            use DummyObjectDiscriminants::*;
            let res = match module.parse().expect("invalid module") {
                CoreDeposit => DummyObject::from(object.parse::<CoreDepositObject>()?),
                Governance => DummyObject::from(object.parse::<GovernanceObject>()?),
            };
            Ok(res)
        }
    }
}

pub mod event {
    use serde::{Deserialize, Serialize};

    use deposit::CoreDepositEvent;
    use governance::GovernanceEvent;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        CoreDeposit(CoreDepositEvent),
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
    impl_event_marker!(CoreDepositEvent, CoreDeposit);
}
