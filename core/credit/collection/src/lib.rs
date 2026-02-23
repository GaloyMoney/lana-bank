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
use core_time_events::CoreTimeEvent;
use es_entity::clock::ClockHandle;
use obix::out::{Outbox, OutboxEventJobConfig, OutboxEventMarker};

pub use error::CoreCreditCollectionError;
pub use obligation::{
    NewObligation, Obligation, ObligationEvent, Obligations, error::ObligationError,
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

use ledger::CollectionLedger;
pub use ledger::error::CollectionLedgerError;

use obligation::jobs::end_of_day::{OBLIGATION_END_OF_DAY, ObligationEndOfDayHandler};

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
    pub fn obligations(&self) -> &Obligations<Perms, E> {
        self.obligations.as_ref()
    }

    pub fn payments(&self) -> &Payments<Perms, E> {
        self.payments.as_ref()
    }
}

impl<Perms, E> CoreCreditCollection<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent> + OutboxEventMarker<CoreTimeEvent>,
{
    pub async fn init(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        cala: &cala_ledger::CalaLedger,
        journal_id: cala_ledger::JournalId,
        payments_made_omnibus_account_id: CalaAccountId,
        jobs: &mut job::Jobs,
        publisher: &CollectionPublisher<E>,
        outbox: &Outbox<E>,
        clock: ClockHandle,
    ) -> Result<Self, CoreCreditCollectionError> {
        let ledger =
            CollectionLedger::init(cala, journal_id, payments_made_omnibus_account_id).await?;
        let ledger_arc = Arc::new(ledger);

        let obligations = Obligations::new(
            pool,
            authz.clone(),
            ledger_arc.clone(),
            publisher,
            clock.clone(),
        );
        let obligations_arc = Arc::new(obligations);

        outbox
            .register_event_handler(
                jobs,
                OutboxEventJobConfig::new(OBLIGATION_END_OF_DAY),
                ObligationEndOfDayHandler::new(obligations_arc.as_ref()),
            )
            .await?;

        let payments = Payments::new(pool, authz, ledger_arc, clock, publisher);
        let payments_arc = Arc::new(payments);

        Ok(Self {
            obligations: obligations_arc,
            payments: payments_arc,
        })
    }
}
