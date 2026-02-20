#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod obligation;
mod payment;
mod payment_allocation;
pub mod public;

mod error;
mod ledger;
mod primitives;
mod publisher;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use es_entity::clock::ClockHandle;
use obix::out::OutboxEventMarker;

pub use error::CoreCreditCollectionError;
pub use obligation::{
    NewObligation, Obligation, ObligationDefaultedReallocationData, ObligationDueReallocationData,
    ObligationEvent, ObligationOverdueReallocationData, Obligations, error::ObligationError,
};
pub use payment::{Payment, PaymentEvent, PaymentLedgerAccountIds, Payments, error::PaymentError};
pub use payment_allocation::{
    PaymentAllocation, PaymentAllocationEvent, error::PaymentAllocationError,
};
pub use primitives::{
    BalanceUpdateData, BalanceUpdatedSource, BeneficiaryId, CalaAccountId,
    CoreCreditCollectionAction, CoreCreditCollectionObject, ObligationAction, ObligationAllOrOne,
    ObligationId, ObligationReceivableAccountIds, ObligationStatus, ObligationType,
    ObligationsAmounts, PERMISSION_SET_COLLECTION_PAYMENT_DATE, PERMISSION_SET_COLLECTION_VIEWER,
    PERMISSION_SET_COLLECTION_WRITER, PaymentAllocationId, PaymentDetailsForAllocation, PaymentId,
    PaymentSourceAccountId,
};
pub use public::*;
pub use publisher::CollectionPublisher;

pub use ledger::CollectionLedgerOps;
pub use ledger::error::CollectionLedgerError;

pub struct CoreCreditCollection<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
    L: CollectionLedgerOps,
{
    obligations: Arc<Obligations<Perms, E, L>>,
    payments: Arc<Payments<Perms, E, L>>,
}

impl<Perms, E, L> Clone for CoreCreditCollection<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
    L: CollectionLedgerOps,
{
    fn clone(&self) -> Self {
        Self {
            obligations: self.obligations.clone(),
            payments: self.payments.clone(),
        }
    }
}

impl<Perms, E, L> CoreCreditCollection<Perms, E, L>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
    L: CollectionLedgerOps,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        ledger: Arc<L>,
        jobs: &mut job::Jobs,
        publisher: &CollectionPublisher<E>,
        clock: ClockHandle,
    ) -> Self {
        let obligations = Obligations::new(
            pool,
            authz.clone(),
            ledger.clone(),
            jobs,
            publisher,
            clock.clone(),
        );
        let obligations_arc = Arc::new(obligations);

        let payments = Payments::new(pool, authz, ledger, clock, publisher);
        let payments_arc = Arc::new(payments);

        Self {
            obligations: obligations_arc,
            payments: payments_arc,
        }
    }

    pub fn obligations(&self) -> &Obligations<Perms, E, L> {
        self.obligations.as_ref()
    }

    pub fn payments(&self) -> &Payments<Perms, E, L> {
        self.payments.as_ref()
    }
}
