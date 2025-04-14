mod entity;
pub mod error;
mod facility_obligations;
mod payment_allocator;
mod repo;

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use cala_ledger::CalaLedger;
use outbox::OutboxEventMarker;

use crate::{
    event::CoreCreditEvent,
    primitives::{CoreCreditAction, CoreCreditObject, CreditFacilityId, PaymentId, UsdCents},
    publisher::CreditFacilityPublisher,
};

pub use entity::Obligation;
pub(crate) use entity::*;
use error::ObligationError;
pub(crate) use facility_obligations::FacilityObligations;
pub use payment_allocator::*;
pub(crate) use repo::*;

pub struct Obligations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    authz: Perms,
    repo: ObligationRepo<E>,
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
        publisher: &CreditFacilityPublisher<E>,
    ) -> Self {
        let obligation_repo = ObligationRepo::new(pool, publisher);
        Self {
            authz: authz.clone(),
            repo: obligation_repo,
        }
    }

    pub async fn create_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        new_obligation: NewObligation,
    ) -> Result<Obligation, ObligationError> {
        self.repo.create_in_op(db, new_obligation).await
    }

    pub async fn allocate_payment_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        payment_id: PaymentId,
        amount: UsdCents,
        audit_info: AuditInfo,
    ) -> Result<PaymentAllocationResult, ObligationError> {
        let obligations = self.facility_obligations(credit_facility_id).await?;

        let new_allocations = PaymentAllocator::new(payment_id, amount)
            .allocate(obligations.data_for_allocation())?;

        let now = crate::time::now();
        let mut updated_obligations = vec![];
        let mut new_allocations_applied = vec![];
        for mut obligation in obligations {
            if let Some(new_allocation) = new_allocations
                .iter()
                .find(|new_allocation| new_allocation.obligation_id == obligation.id)
            {
                obligation
                    .record_payment(
                        new_allocation.id,
                        new_allocation.amount,
                        now,
                        audit_info.clone(),
                    )
                    .did_execute();
                new_allocations_applied.push(*new_allocation);
                updated_obligations.push(obligation);
            }
        }

        // TODO: remove n+1
        for mut obligation in updated_obligations {
            self.repo.update_in_op(db, &mut obligation).await?;
        }

        Ok(PaymentAllocationResult::new(new_allocations_applied))
    }

    async fn facility_obligations(
        &self,
        credit_facility_id: CreditFacilityId,
    ) -> Result<FacilityObligations, ObligationError> {
        let mut obligations = vec![];
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

        Ok(FacilityObligations::new(credit_facility_id, obligations))
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
