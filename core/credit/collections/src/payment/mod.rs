mod entity;
pub mod error;
mod primitives;
mod repo;

use std::sync::Arc;
use tracing::instrument;

use authz::PermissionCheck;
use core_accounting::LedgerTransactionInitiator;
use es_entity::clock::ClockHandle;
use obix::out::OutboxEventMarker;

use crate::{
    event::CoreCollectionsEvent, ledger::CollectionsLedger, primitives::*, publisher::CollectionsPublisher,
};

pub use entity::Payment;
pub use primitives::PaymentSourceAccountId;

#[cfg(feature = "json-schema")]
pub use entity::PaymentEvent;
pub(super) use entity::*;
use error::PaymentError;
pub(crate) use repo::PaymentRepo;

pub struct Payments<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCollectionsEvent>,
    L: crate::ledger::LedgerOps,
{
    repo: Arc<PaymentRepo<E>>,
    authz: Arc<Perms>,
    ledger: Arc<CollectionsLedger<L>>,
}

impl<Perms, E, L> Clone for Payments<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCollectionsEvent>,
    L: crate::ledger::LedgerOps,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
            ledger: self.ledger.clone(),
        }
    }
}

impl<Perms, E, L> Payments<Perms, E, L>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCollectionsEvent>,
    L: crate::ledger::LedgerOps,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        ledger: Arc<CollectionsLedger<L>>,
        clock: ClockHandle,
        publisher: &CollectionsPublisher<E>,
    ) -> Self {
        let repo = PaymentRepo::new(pool, publisher, clock);

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
    /// to `facility_id`. Upon successful creation, the Payment
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
    #[instrument(name = "collections.payment.record_in_op", skip(self, db))]
    pub(super) async fn record_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        payment_id: PaymentId,
        facility_id: FacilityId,
        payment_ledger_account_ids: PaymentLedgerAccountIds,
        amount: UsdCents,
        effective: chrono::NaiveDate,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<Option<Payment>, PaymentError> {
        let new_payment = NewPayment::builder()
            .id(payment_id)
            .ledger_tx_id(payment_id)
            .amount(amount)
            .facility_id(facility_id)
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
            return Ok(None);
        }

        let payment = self.repo.create_in_op(db, new_payment).await?;

        self.ledger
            .record_payment(db, &payment, initiated_by)
            .await?;

        Ok(Some(payment))
    }

    pub(super) async fn record(
        &self,
        payment_id: PaymentId,
        facility_id: FacilityId,
        payment_ledger_account_ids: PaymentLedgerAccountIds,
        amount: UsdCents,
        effective: chrono::NaiveDate,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<Option<Payment>, PaymentError> {
        let mut db = self.repo.begin_op().await?;
        let res = self
            .record_in_op(
                &mut db,
                payment_id,
                facility_id,
                payment_ledger_account_ids,
                amount,
                effective,
                initiated_by,
            )
            .await?;
        db.commit().await?;

        Ok(res)
    }
}
