mod entity;
pub mod error;
mod repo;

use audit::SystemSubject;
use std::sync::Arc;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use es_entity::clock::ClockHandle;
use obix::out::OutboxEventMarker;

use crate::{
    ledger::CollectionLedger, primitives::*, public::CoreCreditCollectionEvent,
    publisher::CollectionPublisher,
};

pub use entity::Payment;

pub use entity::NewPayment;
pub use entity::PaymentEvent;
pub use entity::PaymentLedgerAccountIds;
use error::PaymentError;
pub use repo::PaymentRepo;

pub struct Payments<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    repo: Arc<PaymentRepo<E>>,
    authz: Arc<Perms>,
    ledger: Arc<CollectionLedger>,
}

impl<Perms, E> Clone for Payments<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
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
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(
        pool: &sqlx::PgPool,
        authz: Arc<Perms>,
        ledger: Arc<CollectionLedger>,
        clock: ClockHandle,
        publisher: &CollectionPublisher<E>,
    ) -> Self {
        let repo = PaymentRepo::new(pool, publisher, clock);

        Self {
            repo: Arc::new(repo),
            authz,
            ledger,
        }
    }

    pub async fn find_by_id(&self, payment_id: PaymentId) -> Result<Option<Payment>, PaymentError> {
        self.repo.maybe_find_by_id(payment_id).await
    }

    /// Attempts to create new Payment entity with `payment_id` linked
    /// to `beneficiary_id`. Upon successful creation, the Payment
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
    #[instrument(name = "collection.payment.record_in_op", skip(self, db))]
    pub async fn record_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        payment_id: PaymentId,
        beneficiary_id: BeneficiaryId,
        payment_ledger_account_ids: PaymentLedgerAccountIds,
        amount: UsdCents,
        effective: chrono::NaiveDate,
        initiated_by: &impl SystemSubject,
    ) -> Result<Option<Payment>, PaymentError> {
        let new_payment = NewPayment::builder()
            .id(payment_id)
            .ledger_tx_id(payment_id)
            .amount(amount)
            .beneficiary_id(beneficiary_id)
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
            .record_payment_in_op(db, &payment, initiated_by)
            .await?;

        Ok(Some(payment))
    }

    pub async fn record(
        &self,
        payment_id: PaymentId,
        beneficiary_id: BeneficiaryId,
        payment_ledger_account_ids: PaymentLedgerAccountIds,
        amount: UsdCents,
        effective: chrono::NaiveDate,
        initiated_by: &impl SystemSubject,
    ) -> Result<Option<Payment>, PaymentError> {
        let mut db = self.repo.begin_op().await?;
        let res = self
            .record_in_op(
                &mut db,
                payment_id,
                beneficiary_id,
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
