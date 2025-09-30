mod entity;
pub mod error;
mod repo;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_custody::{
    CoreCustody, CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject, CustodianId,
};
use core_price::Price;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::Jobs;
use outbox::OutboxEventMarker;
use tracing::instrument;

use crate::{
    Collaterals, CreditFacilityProposal, event::CoreCreditEvent, ledger::*, primitives::*,
};

pub use entity::{NewPendingCreditFacility, PendingCreditFacility, PendingCreditFacilityEvent};
use error::*;
use repo::PendingCreditFacilityRepo;
pub use repo::pending_credit_facility_cursor::*;

pub enum CreditFacilityProposalCompletionOutcome {
    Ignored,
    Completed(PendingCreditFacility),
}

pub struct PendingCreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    repo: PendingCreditFacilityRepo<E>,
    custody: CoreCustody<Perms, E>,
    collaterals: Collaterals<Perms, E>,
    authz: Perms,
    jobs: Jobs,
    price: Price,
    ledger: CreditLedger,
    governance: Governance<Perms, E>,
}
impl<Perms, E> Clone for PendingCreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            custody: self.custody.clone(),
            collaterals: self.collaterals.clone(),
            authz: self.authz.clone(),
            jobs: self.jobs.clone(),
            price: self.price.clone(),
            ledger: self.ledger.clone(),
            governance: self.governance.clone(),
        }
    }
}

impl<Perms, E> PendingCreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        custody: &CoreCustody<Perms, E>,
        collaterals: &Collaterals<Perms, E>,
        authz: &Perms,
        jobs: &Jobs,
        ledger: &CreditLedger,
        price: &Price,
        publisher: &crate::CreditFacilityPublisher<E>,
        governance: &Governance<Perms, E>,
    ) -> Result<Self, PendingCreditFacilityError> {
        let repo = PendingCreditFacilityRepo::new(pool, publisher);
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
            custody: custody.clone(),
            collaterals: collaterals.clone(),
            ledger: ledger.clone(),
            jobs: jobs.clone(),
            authz: authz.clone(),
            price: price.clone(),
            governance: governance.clone(),
        })
    }

    pub(super) async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, PendingCreditFacilityError> {
        Ok(self.repo.begin_op().await?)
    }

    pub async fn create_in_op(
        &self,
        mut db: es_entity::DbOp<'_>,
        proposal: &CreditFacilityProposal,
    ) -> Result<(), PendingCreditFacilityError> {
        let account_ids = CreditFacilityProposalAccountIds::new();
        let collateral_id = CollateralId::new();
        let id = proposal.id;

        let wallet_id = if let Some(custodian_id) = proposal.custodian_id {
            let custodian_id: CustodianId = custodian_id;

            #[cfg(feature = "mock-custodian")]
            if custodian_id.is_mock_custodian() {
                self.custody.ensure_mock_custodian_in_op(&mut db).await?;
            }

            let wallet = self
                .custody
                .create_wallet_in_op(&mut db, custodian_id, &format!("CF {id}"))
                .await?;

            Some(wallet.id)
        } else {
            None
        };

        let new_pending_facility = NewPendingCreditFacility::builder()
            .id(id)
            .customer_id(proposal.customer_id)
            .customer_type(proposal.customer_type)
            .ledger_tx_id(LedgerTxId::new())
            .account_ids(account_ids)
            .disbursal_credit_account_id(proposal.disbursal_credit_account_id)
            .collateral_id(collateral_id)
            .terms(proposal.terms)
            .amount(proposal.amount)
            .build()
            .expect("could not build new pending credit facility");

        self.collaterals
            .create_in_op(
                &mut db,
                collateral_id,
                id.into(),
                wallet_id,
                account_ids.collateral_account_id,
            )
            .await?;

        let pending_credit_facility = self
            .repo
            .create_in_op(&mut db, new_pending_facility)
            .await?;

        self.ledger
            .handle_facility_proposal_create(db, &pending_credit_facility)
            .await?;

        Ok(())
    }

    #[instrument(
        name = "credit.credit_facility_proposals.complete_in_op",
        skip(self, db)
    )]
    pub(crate) async fn complete_in_op(
        &self,
        db: &mut es_entity::DbOpWithTime<'_>,
        id: PendingCreditFacilityId,
    ) -> Result<CreditFacilityProposalCompletionOutcome, PendingCreditFacilityError> {
        let mut proposal = self.repo.find_by_id(id).await?;

        let price = self.price.usd_cents_per_btc().await?;

        let balances = self
            .ledger
            .get_credit_facility_proposal_balance(proposal.account_ids)
            .await?;

        let Ok(es_entity::Idempotent::Executed(_)) = proposal.complete(balances, price) else {
            return Ok(CreditFacilityProposalCompletionOutcome::Ignored);
        };

        self.repo.update_in_op(db, &mut proposal).await?;

        Ok(CreditFacilityProposalCompletionOutcome::Completed(proposal))
    }

    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    pub(super) async fn update_collateralization_from_events(
        &self,
        id: PendingCreditFacilityId,
    ) -> Result<PendingCreditFacility, PendingCreditFacilityError> {
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
    ) -> Result<(), PendingCreditFacilityError> {
        let price = self.price.usd_cents_per_btc().await?;
        let mut has_next_page = true;
        let mut after: Option<PendingCreditFacilitiesByCollateralizationRatioCursor> = None;
        while has_next_page {
            let mut credit_facility_proposals = self
                .repo
                .list_by_collateralization_ratio(
                    es_entity::PaginatedQueryArgs::<
                        PendingCreditFacilitiesByCollateralizationRatioCursor,
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
        query: es_entity::PaginatedQueryArgs<PendingCreditFacilitiesByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<
            PendingCreditFacility,
            PendingCreditFacilitiesByCreatedAtCursor,
        >,
        PendingCreditFacilityError,
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
    ) -> Result<Vec<PendingCreditFacility>, PendingCreditFacilityError> {
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
    pub async fn find_all<T: From<PendingCreditFacility>>(
        &self,
        ids: &[PendingCreditFacilityId],
    ) -> Result<std::collections::HashMap<PendingCreditFacilityId, T>, PendingCreditFacilityError>
    {
        self.repo.find_all(ids).await
    }

    pub(crate) async fn find_by_id_without_audit(
        &self,
        id: impl Into<PendingCreditFacilityId> + std::fmt::Debug,
    ) -> Result<PendingCreditFacility, PendingCreditFacilityError> {
        self.repo.find_by_id(id.into()).await
    }

    #[instrument(name = "credit.credit_facility_proposals.find_by_id", skip(self, sub))]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        id: impl Into<PendingCreditFacilityId> + std::fmt::Debug,
    ) -> Result<Option<PendingCreditFacility>, PendingCreditFacilityError> {
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
        id: impl Into<PendingCreditFacilityId> + std::fmt::Debug,
    ) -> Result<Satoshis, PendingCreditFacilityError> {
        let id = id.into();
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(id.into()),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;

        let pending_credit_facility = self.repo.find_by_id(id).await?;

        let collateral = self
            .ledger
            .get_collateral_for_pending_facility(
                pending_credit_facility.account_ids.collateral_account_id,
            )
            .await?;

        Ok(collateral)
    }
}
