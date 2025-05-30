mod entity;
pub mod error;
mod repo;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use outbox::OutboxEventMarker;

pub use entity::PaymentAllocation;
pub(super) use entity::*;
use error::PaymentAllocationError;
pub(super) use repo::*;

use crate::{
    primitives::PaymentAllocationId, publisher::CreditFacilityPublisher, CoreCreditAction,
    CoreCreditEvent, CoreCreditObject,
};

pub struct PaymentAllocations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    repo: PaymentAllocationRepo<E>,
    authz: Perms,
}

impl<Perms, E> Clone for PaymentAllocations<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
        }
    }
}

impl<Perms, E> PaymentAllocations<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(pool: &sqlx::PgPool, authz: &Perms, publisher: &CreditFacilityPublisher<E>) -> Self {
        let repo = PaymentAllocationRepo::new(pool, publisher);
        Self {
            repo,
            authz: authz.clone(),
        }
    }

    pub(super) async fn find_by_id_without_audit(
        &self,
        payment_allocation_id: impl Into<PaymentAllocationId> + std::fmt::Debug,
    ) -> Result<PaymentAllocation, PaymentAllocationError> {
        self.repo.find_by_id(payment_allocation_id.into()).await
    }

    #[instrument(name = "core_credit.payment_allocation.find_by_id", skip(self), err)]
    pub async fn find_by_id(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        payment_allocation_id: impl Into<PaymentAllocationId> + std::fmt::Debug,
    ) -> Result<PaymentAllocation, PaymentAllocationError> {
        let payment_allocation = self.repo.find_by_id(payment_allocation_id.into()).await?;

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
