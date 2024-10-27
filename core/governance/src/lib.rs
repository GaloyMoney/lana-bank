#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod committee;
pub mod error;
mod policy;
mod primitives;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;

pub use committee::*;
use error::*;
pub use policy::*;
pub use primitives::*;

#[derive(Clone)]
pub struct Governance<Perms>
where
    Perms: PermissionCheck,
{
    pool: sqlx::PgPool,
    committee_repo: CommitteeRepo,
    policy_repo: PolicyRepo,
    authz: Perms,
}

impl<Perms> Governance<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<GovernanceObject>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms) -> Self {
        let committee_repo = CommitteeRepo::new(pool);
        let policy_repo = PolicyRepo::new(pool);

        Self {
            pool: pool.clone(),
            committee_repo,
            policy_repo,
            authz: authz.clone(),
        }
    }

    #[instrument(name = "governance.create_policy", skip(self), err)]
    pub async fn create_policy(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        process_type: ApprovalProcessType,
        rules: ApprovalRules,
        committee_id: CommitteeId,
    ) -> Result<Policy, GovernanceError> {
        let audit_info = self
            .authz
            .evaluate_permission(
                sub,
                GovernanceObject::Policy(PolicyAllOrOne::All),
                g_action(PolicyAction::Create),
                true,
            )
            .await?
            .expect("audit info missing");

        let new_policy = NewPolicy::builder()
            .id(PolicyId::new())
            .process_type(process_type)
            .committee_id(committee_id)
            .rules(rules)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new policy");

        let policy = self.policy_repo.create_in_tx(db, new_policy).await?;
        Ok(policy)
    }

    #[instrument(name = "governance.create_committee", skip(self), err)]
    pub async fn create_committee(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: String,
    ) -> Result<Committee, GovernanceError> {
        let audit_info = self
            .authz
            .evaluate_permission(
                sub,
                GovernanceObject::Committee(CommitteeAllOrOne::All),
                g_action(CommitteeAction::Create),
                true,
            )
            .await?
            .expect("audit info missing");

        let new_committee = NewCommittee::builder()
            .id(CommitteeId::new())
            .name(name)
            .audit_info(audit_info)
            .build()
            .expect("Could not build new committee");

        let mut db = self.pool.begin().await?;
        let committee = self
            .committee_repo
            .create_in_tx(&mut db, new_committee)
            .await?;
        db.commit().await?;
        Ok(committee)
    }
}
