mod entity;
pub mod error;
mod repo;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_price::Price;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::Jobs;
use outbox::OutboxEventMarker;
use tracing::instrument;

use crate::{event::CoreCreditEvent, ledger::CreditLedger, primitives::*};

pub use entity::{NewCreditFacilityProposal, CreditFacilityProposal, CreditFacilityProposalEvent};
use error::*;
use repo::CreditFacilityProposalRepo;


pub struct CreditFacilityProposals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    repo: CreditFacilityProposalRepo<E>,
    authz: Perms,
    jobs: Jobs,
    price: Price,
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
            price: self.price.clone(),
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
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: &Perms,
        jobs: &Jobs,
        ledger: &CreditLedger,
        price: &Price,
        publisher: &crate::CreditFacilityPublisher<E>,
        governance: &Governance<Perms, E>,
    ) -> Result<Self, CreditFacilityProposalError> {
        let repo = CreditFacilityProposalRepo::new(pool, publisher);
        match governance
            .init_policy(crate::APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS)
            .await
        {
            Err(governance::error::GovernanceError::PolicyError(
                governance::policy_error::PolicyError::DuplicateApprovalProcessType,
            )) => (),
            Err(e) => return Err(e.into()),
            _ => (),
        }

        Ok(Self {
            repo,
            ledger: ledger.clone(),
            jobs: jobs.clone(),
            authz: authz.clone(),
            price: price.clone(),
            governance: governance.clone(),
        })
    }

    pub(super) async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, CreditFacilityProposalError> {
        Ok(self.repo.begin_op().await?)
    }

}
