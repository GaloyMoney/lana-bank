mod entity;
pub mod error;
mod primitives;
mod repo;

use std::sync::Arc;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_accounting::LedgerTransactionInitiator;
use obix::out::OutboxEventMarker;

use crate::{
    event::CoreCreditEvent, ledger::CreditLedger, primitives::*, publisher::CreditFacilityPublisher,
};

pub use entity::Payment;
pub use primitives::PaymentSourceAccountId;

#[cfg(feature = "json-schema")]
pub use entity::PaymentEvent;
pub(super) use entity::*;
use error::PaymentError;
pub(crate) use repo::PaymentRepo;

pub struct Payments<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    repo: Arc<PaymentRepo<E>>,
    authz: Arc<Perms>,
    ledger: Arc<CreditLedger>,
}

impl<Perms, E> Clone for Payments<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
            ledger: self.ledger.clone(),
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
        authz: Arc<Perms>,
        ledger: Arc<CreditLedger>,
        publisher: &CreditFacilityPublisher<E>,
    ) -> Self {
        let repo = PaymentRepo::new(pool, publisher);

        Self {
            repo: Arc::new(repo),
            authz,
            ledger,
        }
    }

    pub(super) async fn find_by_id(
        &self,
        payment_id: PaymentId,
    ) -> Result<Option<Payment>, PaymentError> {
        self.repo.maybe_find_by_id(payment_id).await
    }

    /// Attempts to create new Payment entity with `payment_id` linked
    /// to `credit_facility_id`. Upon successful creation, the Payment
    /// is recorded in ledger by transferring `amount` from
    /// `payment_source_account_id` to `payment_holding_account_id`
    /// with `effective` date.
    ///
    /// Returns `Some` if the new entity was created
    /// (i. e. `payment_id` was not previously used) and funds
    /// transferred, otherwise returns `None` (in which case no other
    /// operation was performed).
    ///
    /// # Idempotency
    ///
    /// Idempotent via `payment_id`.
    #[instrument(name = "core.credit.payment.record_in_op", skip(self, db))]
    pub(super) async fn record_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        payment_id: PaymentId,
        credit_facility_id: CreditFacilityId,
        payment_ledger_account_ids: PaymentLedgerAccountIds,
        amount: UsdCents,
        effective: chrono::NaiveDate,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<Option<Payment>, PaymentError> {
        let new_payment = NewPayment::builder()
            .id(payment_id)
            .ledger_tx_id(payment_id)
            .amount(amount)
            .credit_facility_id(credit_facility_id)
            .payment_ledger_account_ids(payment_ledger_account_ids)
            .effective(effective)
            .build()
            .expect("could not build new payment");

        if self
            .repo
            .maybe_find_by_id_in_op(&mut *db, payment_id)
            .await?
            .is_some()
        {
            Ok(None)
        } else {
            let payment = self.repo.create_in_op(db, new_payment).await?;
            self.ledger
                .record_payment(db, &payment, initiated_by)
                .await?;
            Ok(Some(payment))
        }
    }
}
