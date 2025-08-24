mod entity;
pub mod error;
mod repo;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_price::Price;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::Jobs;
use outbox::OutboxEventMarker;

use crate::{event::CoreCreditEvent, ledger::CreditLedger, primitives::*};

pub use entity::{CreditFacilityProposal, CreditFacilityProposalEvent, NewCreditFacilityProposal};
use error::*;
use repo::{CreditFacilityProposalRepo, credit_facility_proposal_cursor::*};
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
                crate::APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS,
            )
            .await?;
        self.repo.create_in_op(db, new_proposal).await
    }

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

    #[es_entity::retry_on_concurrent_modification(any_error = true)]
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

            for facility in credit_facility_proposals.entities.iter_mut() {
                // if facility.status() == CreditFacilityStatus::Closed {
                //     continue;
                // } // TODO: handle this case when we have status fn
                let balances = self
                    .ledger
                    .get_credit_facility_proposal_balance(facility.account_ids)
                    .await?;
                if facility
                    .update_collateralization(price, balances)
                    .did_execute()
                {
                    self.repo.update_in_op(&mut op, facility).await?;
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
}
