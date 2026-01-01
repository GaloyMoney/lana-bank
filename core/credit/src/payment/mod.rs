mod entity;
pub mod error;
mod primitives;
mod repo;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_accounting::LedgerTransactionInitiator;

use crate::{ledger::CreditLedger, primitives::*};

pub use entity::Payment;
pub use primitives::PaymentSourceAccountId;

#[cfg(feature = "json-schema")]
pub use entity::PaymentEvent;
pub(super) use entity::*;
use error::PaymentError;
pub(super) use repo::*;

pub struct Payments<Perms>
where
    Perms: PermissionCheck,
{
    repo: Arc<PaymentRepo>,
    authz: Arc<Perms>,
    ledger: Arc<CreditLedger>,
}

impl<Perms> Clone for Payments<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
            ledger: self.ledger.clone(),
        }
    }
}

impl<Perms> Payments<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    pub fn new(pool: &sqlx::PgPool, authz: Arc<Perms>, ledger: Arc<CreditLedger>) -> Self {
        let repo = PaymentRepo::new(pool);

        Self {
            repo: Arc::new(repo),
            authz,
            ledger,
        }
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
    pub(super) async fn record_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        payment_id: PaymentId,
        credit_facility_id: CreditFacilityId,
        payment_holding_account_id: CalaAccountId,
        payment_source_account_id: PaymentSourceAccountId,
        amount: UsdCents,
        effective: chrono::NaiveDate,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<Option<Payment>, PaymentError> {
        let new_payment = NewPayment::builder()
            .id(payment_id)
            .ledger_tx_id(payment_id)
            .amount(amount)
            .credit_facility_id(credit_facility_id)
            .payment_holding_account_id(payment_holding_account_id)
            .payment_source_account_id(payment_source_account_id)
            .effective(effective)
            .build()
            .expect("could not build new payment");

        if self.repo.maybe_find_by_id(payment_id).await?.is_some() {
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
