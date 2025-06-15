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
}

impl PolicyRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use audit::{AuditEntryId, AuditInfo};

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
        let pg_host = std::env::var("PG_HOST").unwrap_or("localhost".to_string());
        let pg_con = format!("postgres://user:password@{pg_host}:5433/pg");
        let pool = sqlx::PgPool::connect(&pg_con).await?;
        Ok(pool)
    }

    #[tokio::test]
    async fn unique_per_process_type() -> anyhow::Result<()> {
        let pool = init_pool().await?;
        let repo = PolicyRepo::new(&pool);
        let process_type = ApprovalProcessType::from_owned(uuid::Uuid::new_v4().to_string());

        let new_policy = NewPolicy::builder()
            .id(PolicyId::new())
            .process_type(process_type.clone())
            .rules(crate::ApprovalRules::SystemAutoApprove)
            .audit_info(dummy_audit_info())
            .build()
            .expect("Could not build new policy");
        repo.create(new_policy).await?;

        let new_policy = NewPolicy::builder()
            .id(PolicyId::new())
            .process_type(process_type)
            .rules(crate::ApprovalRules::SystemAutoApprove)
            .audit_info(dummy_audit_info())
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
