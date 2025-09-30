mod entity;
pub mod error;
mod repo;

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::Jobs;
use outbox::OutboxEventMarker;
use tracing::instrument;

use crate::{event::CoreCreditEvent, primitives::*};

pub use entity::{CreditFacilityProposal, CreditFacilityProposalEvent, NewCreditFacilityProposal};
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
            jobs: jobs.clone(),
            authz: authz.clone(),
            governance: governance.clone(),
        })
    }

    pub(super) async fn begin_op(
        &self,
    ) -> Result<es_entity::DbOp<'_>, CreditFacilityProposalError> {
        Ok(self.repo.begin_op().await?)
    }

    #[instrument(
        name = "credit.credit_facility_proposals.create_in_op",
        skip(self, db, new_proposal)
    )]
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
                crate::APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS,
            )
            .await?;
        self.repo.create_in_op(db, new_proposal).await
    }

    #[instrument(name = "credit.credit_facility_proposals.approve", skip(self, db))]
    pub(super) async fn approve(
        &self,
        db: &mut es_entity::DbOp<'_>,
        id: CreditFacilityProposalId,
        approved: bool,
    ) -> Result<CreditFacilityProposal, CreditFacilityProposalError> {
        let mut facility_proposal = self.repo.find_by_id(id).await?;

        if facility_proposal.is_approval_process_concluded() {
            return Ok(facility_proposal);
        }

        if facility_proposal
            .approval_process_concluded(approved)
            .was_ignored()
        {
            return Ok(facility_proposal);
        }

        self.repo.update_in_op(db, &mut facility_proposal).await?;

        Ok(facility_proposal)
    }

    #[instrument(name = "credit.credit_facility_proposals.find_all", skip(self, ids))]
    pub async fn find_all<T: From<CreditFacilityProposal>>(
        &self,
        ids: &[CreditFacilityProposalId],
    ) -> Result<std::collections::HashMap<CreditFacilityProposalId, T>, CreditFacilityProposalError>
    {
        self.repo.find_all(ids).await
    }
}
