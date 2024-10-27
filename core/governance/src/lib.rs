#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod committee;
pub mod error;
mod primitives;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;

pub use committee::*;
use error::*;
pub use primitives::*;

#[derive(Clone)]
pub struct Governance<Perms>
where
    Perms: PermissionCheck,
{
    pool: sqlx::PgPool,
    committee_repo: CommitteeRepo,
    authz: Perms,
}

impl<Perms> Governance<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<GovernanceObject>,
{
    pub async fn init(pool: &sqlx::PgPool, authz: &Perms) -> Result<Self, GovernanceError> {
        let committee_repo = CommitteeRepo::new(pool);

        let governance = Self {
            pool: pool.clone(),
            committee_repo,
            authz: authz.clone(),
        };

        Ok(governance)
    }

    #[instrument(name = "lava.governance.create_committee", skip(self), err)]
    pub async fn create_committee(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        name: String,
    ) -> Result<Committee, GovernanceError> {
        let audit_info = self
            .authz
            .evaluate_permission(
                sub,
                GovernanceObject::Committee,
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
