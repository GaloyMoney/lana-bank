mod entity;
pub mod error;
mod repo;

use std::sync::Arc;

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
pub use repo::credit_facility_proposal_cursor::*;

pub struct CreditFacilityProposals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    repo: Arc<CreditFacilityProposalRepo<E>>,
    authz: Arc<Perms>,
    jobs: Arc<Jobs>,
    governance: Arc<Governance<Perms, E>>,
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
        authz: Arc<Perms>,
        jobs: Arc<Jobs>,
        publisher: &crate::CreditFacilityPublisher<E>,
        governance: Arc<Governance<Perms, E>>,
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
            repo: Arc::new(repo),
            jobs,
            authz,
            governance,
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

    #[instrument(
        name = "credit.credit_facility_proposals.find_by_id",
        skip(self, sub, id),
        err
    )]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<CreditFacilityProposalId> + std::fmt::Debug,
    ) -> Result<Option<CreditFacilityProposal>, CreditFacilityProposalError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(id.into()),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;
        match self.repo.find_by_id(id).await {
            Ok(credit_facility) => Ok(Some(credit_facility)),
            Err(e) if e.was_not_found() => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[instrument(name = "credit.pending_credit_facility.list", skip(self))]
    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<CreditFacilityProposalsByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<
            CreditFacilityProposal,
            CreditFacilityProposalsByCreatedAtCursor,
        >,
        CreditFacilityProposalError,
    > {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_LIST,
            )
            .await?;

        self.repo
            .list_by_created_at(query, es_entity::ListDirection::Descending)
            .await
    }

    #[instrument(
        name = "credit.credit_facility_proposals.list_for_customer_by_created_at",
        skip(self)
    )]
    pub async fn list_for_customer_by_created_at(
        &self,
        _sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<Vec<CreditFacilityProposal>, CreditFacilityProposalError> {
        Ok(self
            .repo
            .list_for_customer_id_by_created_at(
                customer_id.into(),
                Default::default(),
                es_entity::ListDirection::Descending,
            )
            .await?
            .entities)
    }
}
