mod entity;
pub mod error;
mod repo;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::OutboxEventMarker;

use crate::{
    CoreCreditAction, CoreCreditEvent, CoreCreditObject, Obligations, PaymentAllocation,
    PaymentAllocationRepo, primitives::*, publisher::CreditFacilityPublisher,
};

pub use entity::Payment;
pub(super) use entity::*;
use error::PaymentError;
pub(super) use repo::*;

pub struct Payments<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    repo: PaymentRepo,
    payment_allocation_repo: PaymentAllocationRepo<E>,
    authz: Perms,
    obligations: Obligations<Perms, E>,
}

impl<Perms, E> Clone for Payments<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            payment_allocation_repo: self.payment_allocation_repo.clone(),
            authz: self.authz.clone(),
            obligations: self.obligations.clone(),
        }
    }
}

impl<Perms, E> Payments<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: &Perms,
        obligations: &Obligations<Perms, E>,
        publisher: &CreditFacilityPublisher<E>,
    ) -> Self {
        let repo = PaymentRepo::new(pool);
        let payment_allocation_repo = PaymentAllocationRepo::new(pool, publisher);

        Self {
            repo,
            payment_allocation_repo,
            authz: authz.clone(),
            obligations: obligations.clone(),
        }
    }

    pub(super) async fn record_in_op(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        db: &mut es_entity::DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,
        effective: impl Into<chrono::NaiveDate> + std::fmt::Debug + Copy,
    ) -> Result<Vec<PaymentAllocation>, PaymentError> {
        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreCreditObject::all_obligations(),
                CoreCreditAction::OBLIGATION_RECORD_PAYMENT,
            )
            .await?;

        let new_payment = NewPayment::builder()
            .id(PaymentId::new())
            .amount(amount)
            .credit_facility_id(credit_facility_id)
            .audit_info(audit_info.clone())
            .build()
            .expect("could not build new payment");

        let mut payment = self.repo.create_in_op(db, new_payment).await?;

        let res = self
            .obligations
            .allocate_payment_in_op(
                db,
                credit_facility_id,
                payment.id,
                amount,
                effective.into(),
                &audit_info,
            )
            .await?;

        let _ = payment.record_allocated(
            res.disbursed_amount(),
            res.interest_amount(),
            audit_info.clone(),
        );
        self.repo.update_in_op(db, &mut payment).await?;

        let allocations = self
            .payment_allocation_repo
            .create_all_in_op(db, res.allocations)
            .await?;

        Ok(allocations)
    }

    pub(super) async fn find_allocation_by_id_without_audit(
        &self,
        payment_allocation_id: impl Into<PaymentAllocationId> + std::fmt::Debug,
    ) -> Result<PaymentAllocation, PaymentError> {
        let allocation = self
            .payment_allocation_repo
            .find_by_id(payment_allocation_id.into())
            .await?;

        Ok(allocation)
    }

    #[instrument(name = "core_credit.payment.find_allocation_by_id", skip(self), err)]
    pub async fn find_allocation_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        payment_allocation_id: impl Into<PaymentAllocationId> + std::fmt::Debug,
    ) -> Result<PaymentAllocation, PaymentError> {
        let payment_allocation = self
            .payment_allocation_repo
            .find_by_id(payment_allocation_id.into())
            .await?;

        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::credit_facility(payment_allocation.credit_facility_id),
                CoreCreditAction::CREDIT_FACILITY_READ,
            )
            .await?;

        Ok(payment_allocation)
    }
}
