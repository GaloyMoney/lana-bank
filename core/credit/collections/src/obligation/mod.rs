mod entity;
pub mod error;
mod jobs;
mod repo;

use std::sync::Arc;

use tracing::{Span, instrument};
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_accounting::LedgerTransactionInitiator;
use es_entity::clock::ClockHandle;
use obix::out::OutboxEventMarker;

use crate::{
    event::CoreCollectionsEvent,
    ledger::CollectionsLedger,
    payment_allocation::{PaymentAllocation, PaymentAllocationRepo},
    primitives::{
        FacilityId, ObligationId, ObligationDueReallocationData,
        ObligationOverdueReallocationData, PaymentAllocationId,
        PaymentDetailsForAllocation, UsdCents,
    },
    publisher::CollectionsPublisher,
};

pub use entity::Obligation;
use jobs::{obligation_defaulted, obligation_due, obligation_overdue};

#[cfg(feature = "json-schema")]
pub use entity::ObligationEvent;
pub(crate) use entity::*;
use error::ObligationError;
pub(crate) use repo::ObligationRepo;

pub struct Obligations<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCollectionsEvent>,
    L: crate::ledger::LedgerOps + 'static,
{
    authz: Arc<Perms>,
    repo: Arc<ObligationRepo<E>>,
    payment_allocation_repo: Arc<PaymentAllocationRepo<E>>,
    ledger: Arc<CollectionsLedger<L>>,
    obligation_due_job_spawner: obligation_due::ObligationDueJobSpawner<Perms, E>,
    clock: ClockHandle,
}

impl<Perms, E, L> Clone for Obligations<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCollectionsEvent>,
    L: crate::ledger::LedgerOps + 'static,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            repo: self.repo.clone(),
            payment_allocation_repo: self.payment_allocation_repo.clone(),
            ledger: self.ledger.clone(),
            obligation_due_job_spawner: self.obligation_due_job_spawner.clone(),
            clock: self.clock.clone(),
        }
    }
}

impl<Perms, E, L> Obligations<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCollectionsEvent>,
    L: crate::ledger::LedgerOps + 'static,
{
    pub(crate) fn new(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        ledger: Arc<CollectionsLedger<L>>,
        jobs: &mut job::Jobs,
        publisher: &CollectionsPublisher<E>,
        clock: ClockHandle,
    ) -> Self {
        let obligation_repo_arc = Arc::new(ObligationRepo::new(pool, publisher, clock.clone()));
        let payment_allocation_repo = PaymentAllocationRepo::new(pool, publisher, clock.clone());
        let obligation_defaulted_job_spawner = jobs.add_initializer(
            obligation_defaulted::ObligationDefaultedInit::<Perms, E, L>::new(
                ledger.clone(),
                obligation_repo_arc.clone(),
                authz.clone(),
            ),
        );

        let obligation_overdue_job_spawner =
            jobs.add_initializer(obligation_overdue::ObligationOverdueInit::new(
                ledger.clone(),
                obligation_repo_arc.clone(),
                authz.clone(),
                obligation_defaulted_job_spawner.clone(),
            ));

        let obligation_due_job_spawner =
            jobs.add_initializer(obligation_due::ObligationDueInit::new(
                ledger.clone(),
                obligation_repo_arc.clone(),
                authz.clone(),
                obligation_overdue_job_spawner,
                obligation_defaulted_job_spawner,
            ));
        Self {
            authz,
            repo: obligation_repo_arc,
            ledger,
            payment_allocation_repo: Arc::new(payment_allocation_repo),
            obligation_due_job_spawner,
            clock,
        }
    }

    pub async fn begin_op(&self) -> Result<es_entity::DbOp<'static>, ObligationError> {
        Ok(self.repo.begin_op().await?)
    }

    pub async fn create_with_jobs_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        new_obligation: NewObligation,
    ) -> Result<Obligation, ObligationError> {
        let obligation = self.repo.create_in_op(&mut *op, new_obligation).await?;
        self.obligation_due_job_spawner
            .spawn_at_in_op(
                op,
                job::JobId::new(),
                obligation_due::ObligationDueJobConfig::<Perms, E> {
                    obligation_id: obligation.id,
                    effective: obligation.due_at().date_naive(),
                    _phantom: std::marker::PhantomData,
                },
                obligation.due_at(),
            )
            .await?;

        Ok(obligation)
    }

    pub async fn record_overdue_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        id: ObligationId,
        effective: chrono::NaiveDate,
    ) -> Result<(Obligation, Option<ObligationOverdueReallocationData>), ObligationError> {
        let mut obligation = self.repo.find_by_id(id).await?;

        // TODO: Collections authorization to be handled by credit layer
        // self.authz
        //     .audit()
        //     .record_system_entry_in_tx(
        //         op,
        //         CoreCollectionsObject::obligation(id),
        //         CoreCollectionsAction::OBLIGATION_UPDATE_STATUS,
        //     )
        //     .await
        //     .map_err(authz::error::AuthorizationError::from)?;

        let data = if let es_entity::Idempotent::Executed(overdue) =
            obligation.record_overdue(effective)?
        {
            self.repo.update_in_op(op, &mut obligation).await?;
            Some(overdue)
        } else {
            None
        };

        Ok((obligation, data))
    }

    pub async fn record_due_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        id: ObligationId,
        effective: chrono::NaiveDate,
    ) -> Result<(Obligation, Option<ObligationDueReallocationData>), ObligationError> {
        let mut obligation = self.repo.find_by_id_in_op(&mut *op, id).await?;

        // TODO: Collections authorization to be handled by credit layer
        // self.authz
        //     .audit()
        //     .record_system_entry_in_tx(
        //         op,
        //         CoreCollectionsObject::obligation(id),
        //         CoreCollectionsAction::OBLIGATION_UPDATE_STATUS,
        //     )
        //     .await
        //     .map_err(authz::error::AuthorizationError::from)?;

        let data = if let es_entity::Idempotent::Executed(due) = obligation.record_due(effective) {
            self.repo.update_in_op(op, &mut obligation).await?;
            Some(due)
        } else {
            None
        };

        Ok((obligation, data))
    }

    pub async fn find_by_id_without_audit(
        &self,
        id: ObligationId,
    ) -> Result<Obligation, ObligationError> {
        self.repo.find_by_id(id).await
    }

    #[record_error_severity]
    #[instrument(
        name = "collections.obligation.allocate_payment_in_op",
        skip(self, op),
        fields(
            n_new_allocations,
            n_facility_obligations,
            amount_allocated,
            facility_id
        )
    )]
    pub async fn allocate_payment_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        payment_details @ PaymentDetailsForAllocation {
            facility_id,
            amount,
            ..
        }: PaymentDetailsForAllocation,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), ObligationError> {
        let span = Span::current();
        span.record(
            "facility_id",
            tracing::field::display(facility_id),
        );
        let mut obligations = self.facility_obligations(facility_id).await?;
        span.record("n_facility_obligations", obligations.len());

        obligations.sort();

        let mut remaining = amount;
        let mut new_allocations = Vec::new();
        for obligation in obligations.iter_mut() {
            if let es_entity::Idempotent::Executed(new_allocation) =
                obligation.allocate_payment(remaining, payment_details)
            {
                self.repo.update_in_op(op, obligation).await?;
                remaining -= new_allocation.amount;
                new_allocations.push(new_allocation);
                if remaining == UsdCents::ZERO {
                    break;
                }
            }
        }

        span.record("n_new_allocations", new_allocations.len());

        let allocations = self
            .payment_allocation_repo
            .create_all_in_op(op, new_allocations)
            .await?;

        let amount_allocated = allocations.iter().fold(UsdCents::ZERO, |c, a| c + a.amount);
        tracing::Span::current().record(
            "amount_allocated",
            tracing::field::display(amount_allocated),
        );

        self.ledger
            .record_payment_allocations(op, allocations, initiated_by)
            .await?;

        Ok(())
    }

    pub(super) async fn find_allocation_by_id_without_audit(
        &self,
        allocation_id: impl Into<PaymentAllocationId> + std::fmt::Debug,
    ) -> Result<PaymentAllocation, ObligationError> {
        Ok(self
            .payment_allocation_repo
            .find_by_id(allocation_id.into())
            .await?)
    }

    #[record_error_severity]
    #[instrument(name = "collections.obligation.find_allocation_by_id", skip(self))]
    pub async fn find_allocation_by_id(
        &self,
        _sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        allocation_id: impl Into<PaymentAllocationId> + std::fmt::Debug,
    ) -> Result<PaymentAllocation, ObligationError> {
        let allocation = self
            .payment_allocation_repo
            .find_by_id(allocation_id.into())
            .await?;

        // TODO: Collections authorization to be handled by credit layer
        // self.authz
        //     .enforce_permission(
        //         sub,
        //         CoreCollectionsObject::facility(allocation.facility_id),
        //         CoreCollectionsAction::FACILITY_READ,
        //     )
        //     .await?;

        Ok(allocation)
    }

    pub async fn check_facility_obligations_status_updated(
        &self,
        facility_id: FacilityId,
    ) -> Result<bool, ObligationError> {
        let obligations = self.facility_obligations(facility_id).await?;
        for obligation in obligations.iter() {
            if !obligation.is_status_up_to_date(self.clock.now()) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    #[record_error_severity]
    #[instrument(
        name = "collections.obligation.facility_obligations",
        skip(self),
        fields(facility_id = %facility_id, n_obligations)
    )]
    async fn facility_obligations(
        &self,
        facility_id: FacilityId,
    ) -> Result<Vec<Obligation>, ObligationError> {
        let mut obligations = Vec::new();
        let mut query = Default::default();
        loop {
            let mut res = self
                .repo
                .list_for_facility_id_by_created_at(
                    facility_id,
                    query,
                    es_entity::ListDirection::Ascending,
                )
                .await?;

            obligations.append(&mut res.entities);

            if let Some(q) = res.into_next_query() {
                query = q;
            } else {
                break;
            };
        }

        Span::current().record("n_obligations", obligations.len());

        Ok(obligations)
    }
}
