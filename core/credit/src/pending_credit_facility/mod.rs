mod entity;
pub mod error;
mod repo;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_custody::{CoreCustody, CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::Price;
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use job::Jobs;
use obix::out::OutboxEventMarker;
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::{
    Collaterals, CreditFacilityProposals,
    credit_facility::NewCreditFacilityBuilder,
    credit_facility_proposal::{CreditFacilityProposal, ProposalApprovalOutcome},
    disbursal::NewDisbursalBuilder,
    event::CoreCreditEvent,
    ledger::*,
    primitives::*,
};

pub use entity::{
    NewCreditFacilityWithInitialDisbursal, NewPendingCreditFacility,
    NewPendingCreditFacilityBuilder, PendingCreditFacility, PendingCreditFacilityEvent,
};
use error::*;
use repo::PendingCreditFacilityRepo;
pub use repo::pending_credit_facility_cursor::*;

pub enum PendingCreditFacilityCompletionOutcome {
    Ignored,
    Completed {
        new_credit_facility: NewCreditFacilityBuilder,
        initial_disbursal: Option<NewDisbursalBuilder>,
    },
}

pub struct PendingCreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    repo: Arc<PendingCreditFacilityRepo<E>>,
    proposals: Arc<CreditFacilityProposals<Perms, E>>,
    custody: Arc<CoreCustody<Perms, E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    authz: Arc<Perms>,
    jobs: Arc<Jobs>,
    price: Arc<Price>,
    ledger: Arc<CreditLedger>,
    governance: Arc<Governance<Perms, E>>,
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
            proposals: self.proposals.clone(),
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
    pub fn new(
        pool: &sqlx::PgPool,
        proposals: Arc<CreditFacilityProposals<Perms, E>>,
        custody: Arc<CoreCustody<Perms, E>>,
        collaterals: Arc<Collaterals<Perms, E>>,
        authz: Arc<Perms>,
        jobs: Arc<Jobs>,
        ledger: Arc<CreditLedger>,
        price: Arc<Price>,
        publisher: &crate::CreditFacilityPublisher<E>,
        governance: Arc<Governance<Perms, E>>,
    ) -> Self {
        let repo = PendingCreditFacilityRepo::new(pool, publisher);

        Self {
            repo: Arc::new(repo),
            proposals,
            custody,
            collaterals,
            authz,
            jobs,
            price,
            ledger,
            governance,
        }
    }

    pub(super) async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, PendingCreditFacilityError> {
        Ok(self.repo.begin_op().await?)
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.pending_credit_facility.transition_from_proposal",
        skip(self, credit_facility_proposal_id),
        fields(pending_credit_facility_id = tracing::field::Empty, credit_facility_proposal_id = tracing::field::Empty)
    )]
    pub async fn transition_from_proposal(
        &self,
        credit_facility_proposal_id: impl Into<CreditFacilityProposalId> + std::fmt::Debug,
        approved: bool,
    ) -> Result<Option<CreditFacilityProposal>, PendingCreditFacilityError> {
        let mut db = self.repo.begin_op().await?;

        let id = credit_facility_proposal_id.into();
        tracing::Span::current()
            .record("credit_facility_proposal_id", tracing::field::display(&id));

        match self.proposals.approve_in_op(&mut db, id, approved).await? {
            ProposalApprovalOutcome::AlreadyApplied => Ok(None),
            ProposalApprovalOutcome::Rejected(proposal) => {
                db.commit().await?;
                Ok(Some(proposal))
            }
            ProposalApprovalOutcome::Approved {
                new_pending_facility,
                custodian_id,
                proposal,
            } => {
                let wallet_id = if let Some(custodian_id) = custodian_id {
                    #[cfg(feature = "mock-custodian")]
                    if custodian_id.is_mock_custodian() {
                        self.custody.ensure_mock_custodian_in_op(&mut db).await?;
                    }

                    let wallet = self
                        .custody
                        .create_wallet_in_op(
                            &mut db,
                            custodian_id,
                            &format!("CF {}", new_pending_facility.id),
                        )
                        .await?;

                    Some(wallet.id)
                } else {
                    None
                };

                self.collaterals
                    .create_in_op(
                        &mut db,
                        new_pending_facility.collateral_id,
                        new_pending_facility.id,
                        wallet_id,
                        new_pending_facility.account_ids.collateral_account_id,
                    )
                    .await?;

                tracing::Span::current().record(
                    "pending_credit_facility_id",
                    tracing::field::display(&new_pending_facility.id),
                );
                let pending_credit_facility = self
                    .repo
                    .create_in_op(&mut db, new_pending_facility)
                    .await?;

                self.ledger
                    .handle_pending_facility_creation(
                        &mut db,
                        &pending_credit_facility,
                        core_accounting::LedgerTransactionInitiator::System,
                    )
                    .await?;

                db.commit().await?;

                Ok(Some(proposal))
            }
        }
    }

    #[record_error_severity]
    #[instrument(name = "credit.pending_credit_facility.complete_in_op",
        skip(self, db),
        fields(pending_credit_facility_id = tracing::field::display(&pending_credit_facility_id)))
    ]
    pub(crate) async fn complete_in_op(
        &self,
        db: &mut es_entity::DbOpWithTime<'_>,
        pending_credit_facility_id: PendingCreditFacilityId,
    ) -> Result<PendingCreditFacilityCompletionOutcome, PendingCreditFacilityError> {
        let mut pending_facility = self.repo.find_by_id(pending_credit_facility_id).await?;

        let price = self.price.usd_cents_per_btc().await;

        let balances = self
            .ledger
            .get_pending_credit_facility_balance(pending_facility.account_ids)
            .await?;

        match pending_facility.complete(balances, price, crate::time::now()) {
            Ok(es_entity::Idempotent::Executed(NewCreditFacilityWithInitialDisbursal {
                new_credit_facility,
                initial_disbursal,
            })) => {
                self.repo.update_in_op(db, &mut pending_facility).await?;

                Ok(PendingCreditFacilityCompletionOutcome::Completed {
                    new_credit_facility,
                    initial_disbursal,
                })
            }
            Ok(es_entity::Idempotent::AlreadyApplied)
            | Err(PendingCreditFacilityError::BelowMarginLimit) => {
                Ok(PendingCreditFacilityCompletionOutcome::Ignored)
            }
            Err(e) => Err(e),
        }
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.pending_credit_facility.update_collateralization_from_events",
        skip(self)
    )]
    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    pub(super) async fn update_collateralization_from_events(
        &self,
        id: PendingCreditFacilityId,
    ) -> Result<PendingCreditFacility, PendingCreditFacilityError> {
        let mut op = self.repo.begin_op().await?;
        let mut pending_facility = self.repo.find_by_id_in_op(&mut op, id).await?;

        tracing::Span::current().record(
            "pending_credit_facility_id",
            pending_facility.id.to_string(),
        );

        let balances = self
            .ledger
            .get_pending_credit_facility_balance(pending_facility.account_ids)
            .await?;

        let price = self.price.usd_cents_per_btc().await;

        if pending_facility
            .update_collateralization(price, balances)
            .did_execute()
        {
            self.repo
                .update_in_op(&mut op, &mut pending_facility)
                .await?;

            op.commit().await?;
        }
        Ok(pending_facility)
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.update_collateralization_from_price_event",
        skip(self)
    )]
    pub(super) async fn update_collateralization_from_price_event(
        &self,
        price: PriceOfOneBTC,
    ) -> Result<(), PendingCreditFacilityError> {
        let mut has_next_page = true;
        let mut after: Option<PendingCreditFacilitiesByCollateralizationRatioCursor> = None;
        while has_next_page {
            let mut pending_credit_facilities = self
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
                pending_credit_facilities.end_cursor,
                pending_credit_facilities.has_next_page,
            );
            let mut op = self.repo.begin_op().await?;

            let mut at_least_one = false;

            for pending_facility in pending_credit_facilities.entities.iter_mut() {
                tracing::Span::current().record(
                    "pending_credit_facility_id",
                    pending_facility.id.to_string(),
                );

                if pending_facility.status() == PendingCreditFacilityStatus::Completed {
                    continue;
                }
                let balances = self
                    .ledger
                    .get_pending_credit_facility_balance(pending_facility.account_ids)
                    .await?;
                if pending_facility
                    .update_collateralization(price, balances)
                    .did_execute()
                {
                    self.repo.update_in_op(&mut op, pending_facility).await?;
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

    #[record_error_severity]
    #[instrument(name = "credit.pending_credit_facility.list", skip(self))]
    pub async fn list(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        query: es_entity::PaginatedQueryArgs<PendingCreditFacilitiesByCreatedAtCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<
            PendingCreditFacility,
            PendingCreditFacilitiesByCreatedAtCursor,
        >,
        PendingCreditFacilityError,
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
        name = "credit.pending_credit_facility.list_for_customer_by_created_at",
        skip(self)
    )]
    pub async fn list_for_customer_by_created_at(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        customer_id: impl Into<crate::primitives::CustomerId> + std::fmt::Debug,
    ) -> Result<Vec<PendingCreditFacility>, PendingCreditFacilityError> {
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

    #[record_error_severity]
    #[instrument(name = "credit.pending_credit_facility.find_all", skip(self, ids))]
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

    #[record_error_severity]
    #[instrument(
        name = "credit.pending_credit_facility.find_by_id",
        skip(self, sub, pending_credit_facility_id)
        fields(pending_credit_facility_id = tracing::field::Empty)
    )]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        pending_credit_facility_id: impl Into<PendingCreditFacilityId> + std::fmt::Debug,
    ) -> Result<Option<PendingCreditFacility>, PendingCreditFacilityError> {
        let id = pending_credit_facility_id.into();
        tracing::Span::current().record("pending_credit_facility_id", tracing::field::display(&id));

        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(id.into()),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;

        self.repo.maybe_find_by_id(id).await
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
