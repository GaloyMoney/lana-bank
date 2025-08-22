mod entity;
pub mod error;
mod repo;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_price::Price;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::{JobId, Jobs};
use outbox::OutboxEventMarker;

use crate::{event::CoreCreditEvent, ledger::CreditLedger, primitives::*};

use entity::*;
pub use entity::{CreditFacilityProposal, NewCreditFacilityProposal};
use error::*;
use repo::*;
pub struct CreditFacilityProposals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    repo: CreditFacilityProposalRepo<E>,
    authz: Perms,
    jobs: Jobs,
    ledger: CreditLedger,
    governance: Governance<Perms, E>,
}

impl<Perms, E> Clone for CreditFacilityProposals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
            jobs: self.jobs.clone(),
            ledger: self.ledger.clone(),
            governance: self.governance.clone(),
        }
    }
}

impl<Perms, E> CreditFacilityProposals<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    pub async fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        jobs: &Jobs,
        ledger: &CreditLedger,
        publisher: &crate::CreditFacilityPublisher<E>,
        governance: &Governance<Perms, E>,
    ) -> Self {
        let repo = CreditFacilityProposalRepo::new(pool, publisher);
        let _ = governance
            .init_policy(crate::APPROVE_CREDIT_FACILITY_PROCESS)
            .await;

        Self {
            repo,
            ledger: ledger.clone(),
            jobs: jobs.clone(),
            authz: authz.clone(),
            governance: governance.clone(),
        }
    }

    pub(super) async fn begin_op(
        &self,
    ) -> Result<es_entity::DbOp<'_>, CreditFacilityProposalError> {
        Ok(self.repo.begin_op().await?)
    }

    pub(super) async fn create_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        new_proposal: NewCreditFacilityProposal,
    ) -> Result<CreditFacilityProposal, CreditFacilityProposalError> {
        self.governance
            .start_process(
                db,
                new_proposal.id,
                new_proposal.id.to_string(),
                crate::APPROVE_CREDIT_FACILITY_PROCESS,
            )
            .await?;
        self.repo.create_in_op(db, new_proposal).await
    }
}
