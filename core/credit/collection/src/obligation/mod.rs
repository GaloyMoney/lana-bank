mod entity;
pub mod error;
mod jobs;
mod repo;

use audit::SystemSubject;
use std::sync::Arc;

use tracing::{Span, instrument};
use tracing_macros::record_error_severity;

use audit::AuditSvc;
use authz::PermissionCheck;
use es_entity::clock::ClockHandle;
use obix::out::OutboxEventMarker;

use crate::{
    ledger::CollectionLedger,
    payment_allocation::{PaymentAllocation, PaymentAllocationRepo},
    primitives::*,
    public::CoreCreditCollectionEvent,
    publisher::CollectionPublisher,
};

pub use entity::Obligation;
use jobs::{obligation_defaulted, obligation_due, obligation_overdue};

pub use entity::NewObligation;
pub(crate) use entity::ObligationDefaultedReallocationData;
pub(crate) use entity::ObligationDueReallocationData;
pub use entity::ObligationEvent;
pub(crate) use entity::ObligationOverdueReallocationData;
use error::ObligationError;
pub(crate) use repo::{ObligationRepo, ObligationsFilters};

pub struct Obligations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    authz: Arc<Perms>,
    repo: Arc<ObligationRepo<E>>,
    payment_allocation_repo: Arc<PaymentAllocationRepo<E>>,
    ledger: Arc<CollectionLedger>,
    obligation_due_job_spawner: obligation_due::ObligationDueJobSpawner<Perms, E>,
    clock: ClockHandle,
}

impl<Perms, E> Clone for Obligations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
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

impl<Perms, E> Obligations<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        ledger: Arc<CollectionLedger>,
        jobs: &mut job::Jobs,
        publisher: &CollectionPublisher<E>,
        clock: ClockHandle,
    ) -> Self {
        let obligation_repo_arc = Arc::new(ObligationRepo::new(pool, publisher, clock.clone()));
        let payment_allocation_repo = PaymentAllocationRepo::new(pool, publisher, clock.clone());
        let obligation_defaulted_job_spawner = jobs.add_initializer(
            obligation_defaulted::ObligationDefaultedInit::<Perms, E>::new(
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
            n_beneficiary_obligations,
            amount_allocated,
            beneficiary_id
        )
    )]
    pub async fn allocate_payment_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        payment_details @ PaymentDetailsForAllocation {
            beneficiary_id,
            amount,
            ..
        }: PaymentDetailsForAllocation,
        initiated_by: &impl SystemSubject,
    ) -> Result<(), ObligationError> {
        let span = Span::current();
        span.record("beneficiary_id", tracing::field::display(beneficiary_id));
        let mut obligations = self.beneficiary_obligations(beneficiary_id).await?;
        span.record("n_beneficiary_obligations", obligations.len());

        obligations.sort();

        let mut remaining = amount;
        let mut new_allocations = Vec::new();
        for obligation in obligations.iter_mut() {
            if let es_entity::Idempotent::Executed(new_allocation) =
                obligation.allocate_payment(remaining, payment_details)
            {
                remaining -= new_allocation.amount;
                new_allocations.push(new_allocation);
                if remaining == UsdCents::ZERO {
                    break;
                }
            }
        }
        self.repo.update_all_in_op(op, &mut obligations).await?;

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
            .record_payment_allocations_in_op(op, allocations, initiated_by)
            .await?;

        Ok(())
    }

    pub async fn find_allocation_by_id_without_audit(
        &self,
        allocation_id: impl Into<PaymentAllocationId> + std::fmt::Debug,
    ) -> Result<PaymentAllocation, ObligationError> {
        Ok(self
            .payment_allocation_repo
            .find_by_id(allocation_id.into())
            .await?)
    }

    pub async fn find_allocation_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        allocation_id: impl Into<PaymentAllocationId> + std::fmt::Debug + Copy,
    ) -> Result<PaymentAllocation, ObligationError> {
        let allocation = self
            .find_allocation_by_id_without_audit(allocation_id)
            .await?;
        self.authz
            .evaluate_permission(
                sub,
                CoreCreditCollectionObject::obligation(allocation.obligation_id),
                CoreCreditCollectionAction::OBLIGATION_READ,
                true,
            )
            .await?;
        Ok(allocation)
    }

    pub async fn check_beneficiary_obligations_status_updated(
        &self,
        beneficiary_id: BeneficiaryId,
    ) -> Result<bool, ObligationError> {
        let obligations = self.beneficiary_obligations(beneficiary_id).await?;
        for obligation in obligations.iter() {
            if !obligation.is_status_up_to_date(self.clock.now()) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    #[record_error_severity]
    #[instrument(
        name = "collections.obligation.beneficiary_obligations",
        skip(self),
        fields(beneficiary_id = %beneficiary_id, n_obligations)
    )]
    async fn beneficiary_obligations(
        &self,
        beneficiary_id: BeneficiaryId,
    ) -> Result<Vec<Obligation>, ObligationError> {
        let mut obligations = Vec::new();
        let mut query = Default::default();
        loop {
            let mut res = self
                .repo
                .list_for_filters_by_created_at(
                    ObligationsFilters {
                        beneficiary_id: Some(beneficiary_id),
                    },
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
