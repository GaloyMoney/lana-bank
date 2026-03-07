use serde::{Deserialize, Serialize};

use governance::GovernanceEvent;

pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
    let pg_con = std::env::var("PG_CON")?;
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    Ok(pool)
}

/// Removes any "default" committee and its related data from the database.
/// Used for test isolation when testing the "no default committee" scenario.
pub async fn delete_default_committee(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let committee_ids = r#"SELECT id FROM core_committees WHERE name = 'default'"#;
    let policy_ids =
        format!("SELECT id FROM core_policies WHERE committee_id IN ({committee_ids})");
    sqlx::query(&format!(
        "DELETE FROM core_policy_events_rollup WHERE id IN ({policy_ids})"
    ))
    .execute(pool)
    .await?;
    sqlx::query(&format!(
        "DELETE FROM core_policy_events WHERE id IN ({policy_ids})"
    ))
    .execute(pool)
    .await?;
    sqlx::query(&format!(
        "DELETE FROM core_policies WHERE committee_id IN ({committee_ids})"
    ))
    .execute(pool)
    .await?;
    sqlx::query(&format!(
        "DELETE FROM core_committee_events_rollup WHERE id IN ({committee_ids})"
    ))
    .execute(pool)
    .await?;
    sqlx::query(&format!(
        "DELETE FROM core_committee_events WHERE id IN ({committee_ids})"
    ))
    .execute(pool)
    .await?;
    sqlx::query("DELETE FROM core_committees WHERE name = 'default'")
        .execute(pool)
        .await?;
    Ok(())
}

pub mod action {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyAction;

    impl From<governance::GovernanceAction> for DummyAction {
        fn from(_: governance::GovernanceAction) -> Self {
            Self
        }
    }

    impl From<domain_config::DomainConfigAction> for DummyAction {
        fn from(_: domain_config::DomainConfigAction) -> Self {
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
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DummyObject;

    impl From<governance::GovernanceObject> for DummyObject {
        fn from(_: governance::GovernanceObject) -> Self {
            Self
        }
    }

    impl From<domain_config::DomainConfigObject> for DummyObject {
        fn from(_: domain_config::DomainConfigObject) -> Self {
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
    use super::*;

    #[derive(Debug, Serialize, Deserialize, obix::OutboxEvent)]
    #[serde(tag = "module")]
    pub enum DummyEvent {
        Governance(GovernanceEvent),
        #[serde(other)]
        Unknown,
    }
}
