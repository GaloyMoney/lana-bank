mod entity;
pub mod error;
mod jobs;
mod repo;

use std::sync::Arc;

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use core_custody::{CoreCustody, CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::{CorePriceEvent, Price};
use governance::{Governance, GovernanceAction, GovernanceEvent, GovernanceObject};
use obix::out::{Outbox, OutboxEventMarker};
use tracing::instrument;
use tracing_macros::record_error_severity;

use es_entity::clock::ClockHandle;

use crate::{
    Collaterals, CoreCreditEvent, CreditFacilityProposals,
    collateral::ledger::CollateralLedgerAccountIds,
    credit_facility::NewCreditFacilityBuilder,
    credit_facility_proposal::{CreditFacilityProposal, ProposalApprovalOutcome},
    disbursal::NewDisbursalBuilder,
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
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    repo: Arc<PendingCreditFacilityRepo<E>>,
    proposals: Arc<CreditFacilityProposals<Perms, E>>,
    custody: Arc<CoreCustody<Perms, E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    authz: Arc<Perms>,
    price: Arc<Price>,
    ledger: Arc<CreditLedger>,
    governance: Arc<Governance<Perms, E>>,
    clock: ClockHandle,
}
impl<Perms, E> Clone for PendingCreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            proposals: self.proposals.clone(),
            custody: self.custody.clone(),
            collaterals: self.collaterals.clone(),
            authz: self.authz.clone(),
            price: self.price.clone(),
            ledger: self.ledger.clone(),
            governance: self.governance.clone(),
            clock: self.clock.clone(),
        }
    }
}

impl<Perms, E> PendingCreditFacilities<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        proposals: Arc<CreditFacilityProposals<Perms, E>>,
        custody: Arc<CoreCustody<Perms, E>>,
        collaterals: Arc<Collaterals<Perms, E>>,
        authz: Arc<Perms>,
        ledger: Arc<CreditLedger>,
        price: Arc<Price>,
        publisher: &crate::CreditFacilityPublisher<E>,
        governance: Arc<Governance<Perms, E>>,
        jobs: &mut job::Jobs,
        outbox: &Outbox<E>,
        clock: ClockHandle,
    ) -> Result<Self, PendingCreditFacilityError> {
        let repo_arc = Arc::new(PendingCreditFacilityRepo::new(
            pool,
            publisher,
            clock.clone(),
        ));

        let collateral_repo_arc = Arc::new(crate::collateral::repo::CollateralRepo::new(
            pool,
            publisher,
            clock.clone(),
        ));

        let spawner = jobs.add_initializer(
            jobs::collateralization_from_events_for_pending_facility::PendingCreditFacilityCollateralizationFromEventsInit::new(
                outbox,
                repo_arc.clone(),
                collateral_repo_arc,
                price.clone(),
                ledger.clone(),
            ),
        );

        spawner.spawn_unique(job::JobId::new(), jobs::collateralization_from_events_for_pending_facility::PendingCreditFacilityCollateralizationFromEventsJobConfig::<E>{
            _phantom: std::marker::PhantomData
        }).await?;

        Ok(Self {
            repo: repo_arc,
            proposals,
            custody,
            collaterals,
            authz,
            price,
            ledger,
            governance,
            clock,
        })
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
                        CollateralLedgerAccountIds::new(),
                        new_pending_facility.account_ids.into(),
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
                    .handle_pending_facility_creation_in_op(
                        &mut db,
                        &pending_credit_facility,
                        &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(
                            crate::primitives::PENDING_FACILITY_CREATION,
                        ),
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

        let collateral = self
            .collaterals
            .find_by_id_without_audit(pending_facility.collateral_id)
            .await?;
        let collateral_account_id = collateral.account_id();

        let balances = self
            .ledger
            .get_pending_credit_facility_balance(
                pending_facility.account_ids,
                collateral_account_id,
            )
            .await?;

        match pending_facility.complete(balances, price, self.clock.now()) {
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

        let collateral_entity = self
            .collaterals
            .find_by_id_without_audit(pending_credit_facility.collateral_id)
            .await?;
        let collateral_account_id = collateral_entity.account_id();

        let collateral = self
            .ledger
            .get_collateral_for_pending_facility(collateral_account_id)
            .await?;

        Ok(collateral)
    }
}
