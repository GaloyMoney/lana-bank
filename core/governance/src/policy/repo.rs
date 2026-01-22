use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Policy",
    err = "PolicyError",
    columns(
        process_type(ty = "ApprovalProcessType"),
        committee_id(
            ty = "Option<CommitteeId>",
            create(accessor = "committee_id()"),
            update(accessor = "committee_id()")
        ),
    ),
    tbl_prefix = "core"
)]
pub(crate) struct PolicyRepo {
    #[allow(dead_code)]
    pool: PgPool,
    clock: ClockHandle,
}

impl PolicyRepo {
    pub fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            clock,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
        let pg_con = std::env::var("PG_CON").unwrap();
        let pool = sqlx::PgPool::connect(&pg_con).await?;
        Ok(pool)
    }

    #[tokio::test]
    async fn unique_per_process_type() -> anyhow::Result<()> {
        let pool = init_pool().await?;
        let clock = es_entity::clock::ClockHandle::realtime();
        let repo = PolicyRepo::new(&pool, clock);
        let process_type = ApprovalProcessType::from_owned(uuid::Uuid::new_v4().to_string());

        let new_policy = NewPolicy::builder()
            .id(PolicyId::new())
            .process_type(process_type.clone())
            .rules(crate::ApprovalRules::SystemAutoApprove)
            .build()
            .expect("Could not build new policy");
        repo.create(new_policy).await?;

        let new_policy = NewPolicy::builder()
            .id(PolicyId::new())
            .process_type(process_type)
            .rules(crate::ApprovalRules::SystemAutoApprove)
            .build()
            .expect("Could not build new policy");
        let res = repo.create(new_policy).await;
        assert!(matches!(
            res,
            Err(PolicyError::DuplicateApprovalProcessType)
        ));

        Ok(())
    }
}
