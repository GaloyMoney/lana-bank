mod entity;
pub mod error;
mod primitives;
mod repo;

use std::sync::Arc;

use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use es_entity::Idempotent;
use job::{JobId, Jobs};
use outbox::OutboxEventMarker;

use crate::{
    CreditLedger, PaymentAllocation, PaymentAllocationId, PaymentAllocationRepo,
    event::CoreCreditEvent,
    jobs::obligation_due,
    liquidation_process::{LiquidationProcess, LiquidationProcessRepo},
    primitives::{
        CoreCreditAction, CoreCreditObject, CreditFacilityId, ObligationId, PaymentId, UsdCents,
    },
    publisher::CreditFacilityPublisher,
};

pub use entity::Obligation;

#[cfg(feature = "json-schema")]
pub use entity::ObligationEvent;
pub(crate) use entity::*;
use error::ObligationError;
pub use primitives::*;
pub use repo::obligation_cursor;
use repo::*;

pub struct Obligations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    authz: Arc<Perms>,
    repo: Arc<ObligationRepo<E>>,
    liquidation_process_repo: Arc<LiquidationProcessRepo<E>>,
    payment_allocation_repo: Arc<PaymentAllocationRepo<E>>,
    ledger: Arc<CreditLedger>,
    jobs: Arc<Jobs>,
}

impl<Perms, E> Clone for Obligations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            repo: self.repo.clone(),
            liquidation_process_repo: self.liquidation_process_repo.clone(),
            payment_allocation_repo: self.payment_allocation_repo.clone(),
            ledger: self.ledger.clone(),
            jobs: self.jobs.clone(),
        }
    }
}

impl<Perms, E> Obligations<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub(crate) fn new(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        ledger: Arc<CreditLedger>,
        jobs: Arc<Jobs>,
        publisher: &CreditFacilityPublisher<E>,
    ) -> Self {
        let obligation_repo = ObligationRepo::new(pool, publisher);
        let liquidation_process_repo = LiquidationProcessRepo::new(pool, publisher);
        let payment_allocation_repo = PaymentAllocationRepo::new(pool, publisher);
        Self {
            authz,
            repo: Arc::new(obligation_repo),
            liquidation_process_repo: Arc::new(liquidation_process_repo),
            jobs,
            ledger,
            payment_allocation_repo: Arc::new(payment_allocation_repo),
        }
    }

    pub async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, ObligationError> {
        Ok(self.repo.begin_op().await?)
    }

    pub async fn create_with_jobs_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        new_obligation: NewObligation,
    ) -> Result<Obligation, ObligationError> {
        let obligation = self.repo.create_in_op(&mut *op, new_obligation).await?;
        self.jobs
            .create_and_spawn_at_in_op(
                op,
                JobId::new(),
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

        self.authz
            .audit()
            .record_system_entry_in_tx(
                op,
                CoreCreditObject::obligation(id),
                CoreCreditAction::OBLIGATION_UPDATE_STATUS,
            )
            .await
            .map_err(authz::error::AuthorizationError::from)?;

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
        let mut obligation = self.repo.find_by_id(id).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                op,
                CoreCreditObject::obligation(id),
                CoreCreditAction::OBLIGATION_UPDATE_STATUS,
            )
            .await
            .map_err(authz::error::AuthorizationError::from)?;

        let data = if let es_entity::Idempotent::Executed(due) = obligation.record_due(effective) {
            self.repo.update_in_op(op, &mut obligation).await?;
            Some(due)
        } else {
            None
        };

        Ok((obligation, data))
    }

    pub async fn record_defaulted_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        id: ObligationId,
        effective: chrono::NaiveDate,
    ) -> Result<Option<ObligationDefaultedReallocationData>, ObligationError> {
        let mut obligation = self.repo.find_by_id(id).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                op,
                CoreCreditObject::obligation(id),
                CoreCreditAction::OBLIGATION_UPDATE_STATUS,
            )
            .await
            .map_err(authz::error::AuthorizationError::from)?;

        let data = if let es_entity::Idempotent::Executed(defaulted) =
            obligation.record_defaulted(effective)?
        {
            self.repo.update_in_op(op, &mut obligation).await?;
            Some(defaulted)
        } else {
            None
        };

        Ok(data)
    }

    pub async fn start_liquidation_process_in_op(
        &self,
        op: &mut es_entity::DbOp<'_>,
        id: ObligationId,
        effective: chrono::NaiveDate,
    ) -> Result<(Obligation, Option<LiquidationProcess>), ObligationError> {
        let mut obligation = self.repo.find_by_id(id).await?;

        self.authz
            .audit()
            .record_system_entry_in_tx(
                op,
                CoreCreditObject::obligation(id),
                CoreCreditAction::OBLIGATION_UPDATE_STATUS,
            )
            .await
            .map_err(authz::error::AuthorizationError::from)?;

        let liquidation_process = if let Idempotent::Executed(new_liquidation_process) =
            obligation.start_liquidation(effective)
        {
            self.repo.update_in_op(op, &mut obligation).await?;
            let liquidation_process = self
                .liquidation_process_repo
                .create_in_op(op, new_liquidation_process)
                .await?;

            Some(liquidation_process)
        } else {
            None
        };

        Ok((obligation, liquidation_process))
    }

    pub async fn find_by_id_without_audit(
        &self,
        id: ObligationId,
    ) -> Result<Obligation, ObligationError> {
        self.repo.find_by_id(id).await
    }

    #[instrument(
        name = "credit.obligation.allocate_payment_in_op",
        skip(self, op),
        fields(n_new_allocations, n_facility_obligations, amount_allocated)
    )]
    pub async fn allocate_payment_in_op(
        &self,
        mut op: es_entity::DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        payment_id: PaymentId,
        amount: UsdCents,
        effective: chrono::NaiveDate,
    ) -> Result<(), ObligationError> {
        let span = Span::current();
        let mut obligations = self.facility_obligations(credit_facility_id).await?;
        span.record("n_facility_obligations", obligations.len());

        obligations.sort();

        let mut remaining = amount;
        let mut new_allocations = Vec::new();
        for obligation in obligations.iter_mut() {
            if let es_entity::Idempotent::Executed(new_allocation) =
                obligation.allocate_payment(remaining, payment_id, effective)
            {
                self.repo.update_in_op(&mut op, obligation).await?;
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
            .create_all_in_op(&mut op, new_allocations)
            .await?;

        let amount_allocated = allocations.iter().fold(UsdCents::ZERO, |c, a| c + a.amount);
        tracing::Span::current().record(
            "amount_allocated",
            tracing::field::display(amount_allocated),
        );

        self.ledger
            .record_payment_allocations(op, allocations)
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

    #[instrument(name = "core_credit.obligation.find_allocation_by_id", skip(self), err)]
    pub async fn find_allocation_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        allocation_id: impl Into<PaymentAllocationId> + std::fmt::Debug,
    ) -> Result<PaymentAllocation, ObligationError> {
        let allocation = self
            .payment_allocation_repo
            .find_by_id(allocation_id.into())
            .await?;

        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(allocation.credit_facility_id),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;

        Ok(allocation)
    }

    pub async fn check_facility_obligations_status_updated(
        &self,
        credit_facility_id: CreditFacilityId,
    ) -> Result<bool, ObligationError> {
        let obligations = self.facility_obligations(credit_facility_id).await?;
        for obligation in obligations.iter() {
            if !obligation.is_status_up_to_date(crate::time::now()) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn facility_obligations(
        &self,
        credit_facility_id: CreditFacilityId,
    ) -> Result<Vec<Obligation>, ObligationError> {
        let mut obligations = Vec::new();
        let mut query = Default::default();
        loop {
            let mut res = self
                .repo
                .list_for_credit_facility_id_by_created_at(
                    credit_facility_id,
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

        Ok(obligations)
    }
}
