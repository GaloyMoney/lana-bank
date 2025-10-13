mod entity;
pub mod error;
mod repo;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_price::Price;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::Jobs;
use outbox::OutboxEventMarker;
use tracing::instrument;

use crate::{
    credit_facility::NewCreditFacilityBuilder, event::CoreCreditEvent, ledger::CreditLedger,
    primitives::*,
};

pub use entity::{CreditFacilityProposal, CreditFacilityProposalEvent, NewCreditFacilityProposal};
use error::*;
use repo::CreditFacilityProposalRepo;
pub use repo::credit_facility_proposal_cursor::*;

pub enum CreditFacilityProposalCompletionOutcome {
    Ignored,
    Completed(NewCreditFacilityBuilder),
}

pub struct CreditFacilityProposals<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<GovernanceEvent>,
{
    repo: Arc<CreditFacilityProposalRepo<E>>,
    authz: Arc<Perms>,
    jobs: Arc<Jobs>,
    price: Arc<Price>,
    ledger: Arc<CreditLedger>,
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
        authz: Arc<Perms>,
        jobs: Arc<Jobs>,
        ledger: Arc<CreditLedger>,
        price: Arc<Price>,
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
            ledger,
            jobs,
            authz,
            price,
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

    #[es_entity::retry_on_concurrent_modification]
    #[instrument(name = "credit.credit_facility_proposals.approve", skip(self))]
    pub(super) async fn approve(
        &self,
        id: CreditFacilityProposalId,
        approved: bool,
    ) -> Result<CreditFacilityProposal, CreditFacilityProposalError> {
        let mut facility_proposal = self.repo.find_by_id(id).await?;

        if facility_proposal.is_approval_process_concluded() {
            return Ok(facility_proposal);
        }

        let mut op = self.repo.begin_op().await?;

        if facility_proposal
            .approval_process_concluded(approved)
            .was_ignored()
        {
            return Ok(facility_proposal);
        }

        self.repo
            .update_in_op(&mut op, &mut facility_proposal)
            .await?;
        op.commit().await?;

        Ok(facility_proposal)
    }

    #[instrument(
        name = "credit.credit_facility_proposals.complete_in_op",
        skip(self, db)
    )]
    pub(crate) async fn complete_in_op(
        &self,
        db: &mut es_entity::DbOpWithTime<'_>,
        id: CreditFacilityProposalId,
    ) -> Result<CreditFacilityProposalCompletionOutcome, CreditFacilityProposalError> {
        let mut proposal = self.repo.find_by_id(id).await?;

        let price = self.price.usd_cents_per_btc().await?;

        let balances = self
            .ledger
            .get_credit_facility_proposal_balance(proposal.account_ids)
            .await?;

        match proposal.complete(balances, price, crate::time::now()) {
            Ok(es_entity::Idempotent::Executed(new_facility)) => {
                self.repo.update_in_op(db, &mut proposal).await?;

                Ok(CreditFacilityProposalCompletionOutcome::Completed(
                    new_facility,
                ))
            }
            Ok(es_entity::Idempotent::Ignored)
            | Err(CreditFacilityProposalError::BelowMarginLimit)
            | Err(CreditFacilityProposalError::ApprovalInProgress) => {
                Ok(CreditFacilityProposalCompletionOutcome::Ignored)
            }
            Err(e) => Err(e),
        }
    }

    #[es_entity::retry_on_concurrent_modification]
    pub(super) async fn update_collateralization_from_events(
        &self,
        id: CreditFacilityProposalId,
    ) -> Result<CreditFacilityProposal, CreditFacilityProposalError> {
        let mut op = self.repo.begin_op().await?;
        let mut facility_proposal = self.repo.find_by_id_in_op(&mut op, id).await?;

        let balances = self
            .ledger
            .get_credit_facility_proposal_balance(facility_proposal.account_ids)
            .await?;

        let price = self.price.usd_cents_per_btc().await?;

        if facility_proposal
            .update_collateralization(price, balances)
            .did_execute()
        {
            self.repo
                .update_in_op(&mut op, &mut facility_proposal)
                .await?;

            op.commit().await?;
        }
        Ok(facility_proposal)
    }

    pub(super) async fn update_collateralization_from_price(
        &self,
    ) -> Result<(), CreditFacilityProposalError> {
        let price = self.price.usd_cents_per_btc().await?;
        let mut has_next_page = true;
        let mut after: Option<CreditFacilityProposalsByCollateralizationRatioCursor> = None;
        while has_next_page {
            let mut credit_facility_proposals = self
                .repo
                .list_by_collateralization_ratio(
                    es_entity::PaginatedQueryArgs::<
                        CreditFacilityProposalsByCollateralizationRatioCursor,
                    > {
                        first: 10,
                        after,
                    },
                    Default::default(),
                )
                .await?;
            (after, has_next_page) = (
                credit_facility_proposals.end_cursor,
                credit_facility_proposals.has_next_page,
            );
            let mut op = self.repo.begin_op().await?;

            let mut at_least_one = false;

            for proposal in credit_facility_proposals.entities.iter_mut() {
                if proposal.status() == CreditFacilityProposalStatus::Completed {
                    continue;
                }
                let balances = self
                    .ledger
                    .get_credit_facility_proposal_balance(proposal.account_ids)
                    .await?;
                if proposal
                    .update_collateralization(price, balances)
                    .did_execute()
                {
                    self.repo.update_in_op(&mut op, proposal).await?;
                    at_least_one = true;
                }
            }

            if at_least_one {
                op.commit().await?;
            } else {
                break;
            }
        }
        Ok(())
    }

    #[instrument(name = "credit.credit_facility_proposals.list", skip(self))]
    pub async fn list(
        &self,
        _sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<CreditFacilityProposalsByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<
            CreditFacilityProposal,
            CreditFacilityProposalsByCreatedAtCursor,
        >,
        CreditFacilityProposalError,
    > {
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
        customer_id: impl Into<crate::primitives::CustomerId> + std::fmt::Debug,
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

    #[instrument(name = "credit.credit_facility_proposals.find_all", skip(self, ids))]
    pub async fn find_all<T: From<CreditFacilityProposal>>(
        &self,
        ids: &[CreditFacilityProposalId],
    ) -> Result<std::collections::HashMap<CreditFacilityProposalId, T>, CreditFacilityProposalError>
    {
        self.repo.find_all(ids).await
    }

    pub(crate) async fn find_by_id_without_audit(
        &self,
        id: impl Into<CreditFacilityProposalId> + std::fmt::Debug,
    ) -> Result<CreditFacilityProposal, CreditFacilityProposalError> {
        self.repo.find_by_id(id.into()).await
    }

    #[instrument(name = "credit.credit_facility_proposals.find_by_id", skip(self, sub))]
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

    pub async fn collateral(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<CreditFacilityProposalId> + std::fmt::Debug,
    ) -> Result<Satoshis, CreditFacilityProposalError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(id.into()),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;

        let credit_facility_proposal = self.repo.find_by_id(id).await?;

        let collateral = self
            .ledger
            .get_proposal_collateral(credit_facility_proposal.account_ids.collateral_account_id)
            .await?;

        Ok(collateral)
    }
}
