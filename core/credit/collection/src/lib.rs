#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod obligation;
mod payment;
mod payment_allocation;
pub mod public;

mod error;
mod ledger;
mod primitives;
mod publisher;

#[cfg(feature = "json-schema")]
pub use obligation::ObligationEvent;
#[cfg(feature = "json-schema")]
pub use payment::PaymentEvent;
#[cfg(feature = "json-schema")]
pub use payment_allocation::PaymentAllocationEvent;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use es_entity::clock::ClockHandle;
use obix::out::OutboxEventMarker;

pub use error::CoreCreditCollectionError;
pub use obligation::{
    NewObligation, Obligation, Obligations, error::ObligationError, obligation_cursor,
};
pub use payment::{Payment, PaymentLedgerAccountIds, Payments, error::PaymentError};
pub use payment_allocation::{PaymentAllocation, error::PaymentAllocationError};
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

use ledger::CollectionLedger;
pub use ledger::error::CollectionLedgerError;

pub struct CoreCreditCollection<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    obligations: Arc<Obligations<Perms, E>>,
    payments: Arc<Payments<Perms, E>>,
}

impl<Perms, E> Clone for CoreCreditCollection<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    fn clone(&self) -> Self {
        Self {
            obligations: self.obligations.clone(),
            payments: self.payments.clone(),
        }
    }
}

impl<Perms, E> CoreCreditCollection<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        cala: &cala_ledger::CalaLedger,
        journal_id: cala_ledger::JournalId,
        payments_made_omnibus_account_id: CalaAccountId,
        jobs: &mut job::Jobs,
        publisher: &CollectionPublisher<E>,
        clock: ClockHandle,
    ) -> Result<Self, CoreCreditCollectionError> {
        let ledger =
            CollectionLedger::init(cala, journal_id, payments_made_omnibus_account_id).await?;
        let ledger_arc = Arc::new(ledger);

        let obligations = Obligations::new(
            pool,
            authz.clone(),
            ledger_arc.clone(),
            jobs,
            publisher,
            clock.clone(),
        );
        let obligations_arc = Arc::new(obligations);

        let payments = Payments::new(pool, authz, ledger_arc, clock, publisher);
        let payments_arc = Arc::new(payments);

        Ok(Self {
            obligations: obligations_arc,
            payments: payments_arc,
        })
    }

    pub fn obligations(&self) -> &Obligations<Perms, E> {
        self.obligations.as_ref()
    }

    pub fn payments(&self) -> &Payments<Perms, E> {
        self.payments.as_ref()
    }
}
