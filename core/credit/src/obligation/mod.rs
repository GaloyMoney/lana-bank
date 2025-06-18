mod entity;
pub mod error;
// mod payment_allocator;
mod primitives;
mod repo;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use es_entity::Idempotent;
use job::{JobId, Jobs};
use outbox::OutboxEventMarker;

use crate::{
    event::CoreCreditEvent,
    jobs::obligation_due,
    liquidation_process::LiquidationProcessRepo,
    payment_allocation::NewPaymentAllocation,
    primitives::{
        CoreCreditAction, CoreCreditObject, CreditFacilityId, ObligationId, ObligationType,
        PaymentId, UsdCents,
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
    authz: Perms,
    repo: ObligationRepo<E>,
    liquidation_process_repo: LiquidationProcessRepo<E>,
    jobs: Jobs,
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
        authz: &Perms,
        _cala: &CalaLedger,
        jobs: &Jobs,
        publisher: &CreditFacilityPublisher<E>,
    ) -> Self {
        let obligation_repo = ObligationRepo::new(pool, publisher);
        let liquidation_process_repo = LiquidationProcessRepo::new(pool, publisher);
        Self {
            authz: authz.clone(),
            repo: obligation_repo,
            liquidation_process_repo,
            jobs: jobs.clone(),
        }
    }

    pub async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, ObligationError> {
        Ok(self.repo.begin_op().await?)
    }

    pub async fn create_with_jobs_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        new_obligation: NewObligation,
    ) -> Result<Obligation, ObligationError> {
        let obligation = self.repo.create_in_op(db, new_obligation).await?;
        self.jobs
            .create_and_spawn_at_in_op(
                db,
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
        db: &mut es_entity::DbOp<'_>,
        id: ObligationId,
        effective: chrono::NaiveDate,
    ) -> Result<(Obligation, Option<ObligationOverdueReallocationData>), ObligationError> {
        let mut obligation = self.repo.find_by_id(id).await?;

        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::obligation(id),
                CoreCreditAction::OBLIGATION_UPDATE_STATUS,
            )
            .await
            .map_err(authz::error::AuthorizationError::from)?;

        let data = if let es_entity::Idempotent::Executed(overdue) =
            obligation.record_overdue(effective, audit_info)?
        {
            self.repo.update_in_op(db, &mut obligation).await?;
            Some(overdue)
        } else {
            None
        };

        Ok((obligation, data))
    }

    pub async fn record_due_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        id: ObligationId,
        effective: chrono::NaiveDate,
    ) -> Result<(Obligation, Option<ObligationDueReallocationData>), ObligationError> {
        let mut obligation = self.repo.find_by_id(id).await?;

        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::obligation(id),
                CoreCreditAction::OBLIGATION_UPDATE_STATUS,
            )
            .await
            .map_err(authz::error::AuthorizationError::from)?;

        let data = if let es_entity::Idempotent::Executed(due) =
            obligation.record_due(effective, audit_info)
        {
            self.repo.update_in_op(db, &mut obligation).await?;
            Some(due)
        } else {
            None
        };

        Ok((obligation, data))
    }

    pub async fn record_defaulted_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        id: ObligationId,
        effective: chrono::NaiveDate,
    ) -> Result<Option<ObligationDefaultedReallocationData>, ObligationError> {
        let mut obligation = self.repo.find_by_id(id).await?;

        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::obligation(id),
                CoreCreditAction::OBLIGATION_UPDATE_STATUS,
            )
            .await
            .map_err(authz::error::AuthorizationError::from)?;

        let data = if let es_entity::Idempotent::Executed(defaulted) =
            obligation.record_defaulted(effective, audit_info)?
        {
            self.repo.update_in_op(db, &mut obligation).await?;
            Some(defaulted)
        } else {
            None
        };

        Ok(data)
    }

    pub async fn start_liquidation_process_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        id: ObligationId,
    ) -> Result<(), ObligationError> {
        let mut obligation = self.repo.find_by_id(id).await?;

        let audit_info = self
            .authz
            .audit()
            .record_system_entry_in_tx(
                db.tx(),
                CoreCreditObject::obligation(id),
                CoreCreditAction::OBLIGATION_UPDATE_STATUS,
            )
            .await
            .map_err(authz::error::AuthorizationError::from)?;

        if let Idempotent::Executed(new_liquidation_process) =
            obligation.start_liquidation(&audit_info)
        {
            self.repo.update_in_op(db, &mut obligation).await?;
            self.liquidation_process_repo
                .create_in_op(db, new_liquidation_process)
                .await?;
        }

        Ok(())
    }

    pub async fn find_by_id_without_audit(
        &self,
        id: ObligationId,
    ) -> Result<Obligation, ObligationError> {
        self.repo.find_by_id(id).await
    }

    pub async fn allocate_payment_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        payment_id: PaymentId,
        amount: UsdCents,
        effective: chrono::NaiveDate,
        audit_info: &AuditInfo,
    ) -> Result<PaymentAllocationResult, ObligationError> {
        let mut obligations = self.facility_obligations(credit_facility_id).await?;

        obligations.sort();

        let mut remaining = amount;
        let mut new_allocations = Vec::new();
        for obligation in obligations.iter_mut() {
            if let es_entity::Idempotent::Executed(new_allocation) =
                obligation.allocate_payment(remaining, payment_id, effective, audit_info)
            {
                self.repo.update_in_op(db, obligation).await?;
                remaining -= new_allocation.amount;
                new_allocations.push(new_allocation);
                if remaining == UsdCents::ZERO {
                    break;
                }
            }
        }

        Ok(PaymentAllocationResult::new(new_allocations))
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

pub struct PaymentAllocationResult {
    pub allocations: Vec<NewPaymentAllocation>,
}

impl PaymentAllocationResult {
    fn new(allocations: Vec<NewPaymentAllocation>) -> Self {
        Self { allocations }
    }

    pub fn disbursed_amount(&self) -> UsdCents {
        self.allocations
            .iter()
            .fold(UsdCents::from(0), |mut total, allocation| {
                if let NewPaymentAllocation {
                    amount,
                    obligation_type: ObligationType::Disbursal,
                    ..
                } = allocation
                {
                    total += *amount;
                }
                total
            })
    }

    pub fn interest_amount(&self) -> UsdCents {
        self.allocations
            .iter()
            .fold(UsdCents::from(0), |mut total, allocation| {
                if let NewPaymentAllocation {
                    amount,
                    obligation_type: ObligationType::Interest,
                    ..
                } = allocation
                {
                    total += *amount;
                }
                total
            })
    }
}
