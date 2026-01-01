mod entity;
pub mod error;
mod repo;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_custody::CustodianId;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use obix::out::OutboxEventMarker;
use tracing::{Span, instrument};
use tracing_macros::record_error_severity;

use crate::{
    event::CoreCreditEvent, pending_credit_facility::NewPendingCreditFacility, primitives::*,
};

pub use entity::{CreditFacilityProposal, CreditFacilityProposalEvent, NewCreditFacilityProposal};
use error::*;
use repo::CreditFacilityProposalRepo;
pub use repo::credit_facility_proposal_cursor::*;

pub enum ProposalApprovalOutcome {
    Rejected(CreditFacilityProposal),
    Approved {
        new_pending_facility: NewPendingCreditFacility,
        custodian_id: Option<CustodianId>,
        proposal: CreditFacilityProposal,
    },
    AlreadyApplied,
}

pub struct CreditFacilityProposals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    repo: Arc<CreditFacilityProposalRepo<E>>,
    authz: Arc<Perms>,
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
            authz,
            governance,
        })
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility_proposals.create_in_op",
        skip(self, db, new_proposal)
    )]
    pub(super) async fn create_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        new_proposal: NewCreditFacilityProposal,
    ) -> Result<CreditFacilityProposal, CreditFacilityProposalError> {
        self.repo.create_in_op(db, new_proposal).await
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility_proposals.conclude_customer_approval",
        skip(self, credit_facility_proposal_id),
        fields(credit_facility_proposal_id = tracing::field::Empty),
    )]
    pub async fn conclude_customer_approval(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_proposal_id: impl Into<CreditFacilityProposalId> + std::fmt::Debug,
        approved: bool,
    ) -> Result<CreditFacilityProposal, CreditFacilityProposalError> {
        let id = credit_facility_proposal_id.into();
        Span::current().record("credit_facility_proposal_id", tracing::field::display(&id));

        self.authz
            .evaluate_permission(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_CUSTOMER_APPROVE,
                true,
            )
            .await?;

        let mut proposal = self.repo.find_by_id(id).await?;

        match proposal.conclude_customer_approval(approved) {
            es_entity::Idempotent::Executed(_) => {
                let mut db = self.repo.begin_op().await?;

                if approved {
                    self.governance
                        .start_process(
                            &mut db,
                            id,
                            id.to_string(),
                            crate::APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS,
                        )
                        .await?;
                }
                self.repo.update_in_op(&mut db, &mut proposal).await?;

                db.commit().await?;
                Ok(proposal)
            }
            es_entity::Idempotent::AlreadyApplied => Ok(proposal),
        }
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility_proposals.approve_in_op",
        skip(self, db, credit_facility_proposal_id),
        fields(credit_facility_proposal_id = tracing::field::Empty),
    )]
    pub(super) async fn approve_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        credit_facility_proposal_id: impl Into<CreditFacilityProposalId> + std::fmt::Debug,
        approved: bool,
    ) -> Result<ProposalApprovalOutcome, CreditFacilityProposalError> {
        let id = credit_facility_proposal_id.into();
        Span::current().record("credit_facility_proposal_id", tracing::field::display(&id));

        let mut proposal = self.repo.find_by_id(id).await?;
        match proposal.conclude_approval_process(approved)? {
            es_entity::Idempotent::Executed(res) => {
                self.repo.update_in_op(db, &mut proposal).await?;
                Ok(match res {
                    Some((new_pending_facility, custodian_id)) => {
                        ProposalApprovalOutcome::Approved {
                            new_pending_facility,
                            custodian_id,
                            proposal,
                        }
                    }
                    None => ProposalApprovalOutcome::Rejected(proposal),
                })
            }
            es_entity::Idempotent::AlreadyApplied => Ok(ProposalApprovalOutcome::AlreadyApplied),
        }
    }

    #[record_error_severity]
    #[instrument(name = "credit.credit_facility_proposals.find_all", skip(self, ids))]
    pub async fn find_all<T: From<CreditFacilityProposal>>(
        &self,
        ids: &[CreditFacilityProposalId],
    ) -> Result<std::collections::HashMap<CreditFacilityProposalId, T>, CreditFacilityProposalError>
    {
        self.repo.find_all(ids).await
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility_proposals.find_by_id",
        skip(self, sub, credit_facility_proposal_id),
        fields(credit_facility_proposal_id = tracing::field::Empty),
    )]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        credit_facility_proposal_id: impl Into<CreditFacilityProposalId> + std::fmt::Debug,
    ) -> Result<Option<CreditFacilityProposal>, CreditFacilityProposalError> {
        let id = credit_facility_proposal_id.into();
        Span::current().record("credit_facility_proposal_id", tracing::field::display(&id));

        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(id.into()),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;
        self.repo.maybe_find_by_id(id).await
    }

    #[record_error_severity]
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

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility_proposals.list_for_customer_by_created_at",
        skip(self)
    )]
    pub async fn list_for_customer_by_created_at(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<CustomerId> + std::fmt::Debug,
    ) -> Result<Vec<CreditFacilityProposal>, CreditFacilityProposalError> {
        self.authz
            .audit()
            .record_entry(
                sub,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_LIST,
                true,
            )
            .await?;

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
