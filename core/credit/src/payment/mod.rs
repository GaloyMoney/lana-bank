mod entity;
pub mod error;
mod repo;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;

use crate::{ledger::CreditLedger, primitives::*};

pub use entity::Payment;

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

    /// Attempts to create new Payment entity with
    /// `payment_id`. Returns `true` if the new entity was created
    /// (i. e. `payment_id` was not previously used), otherwise
    /// returns `false`.
    pub(super) async fn record_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        new_payment: NewPayment,
    ) -> Result<bool, PaymentError> {
        if self.repo.maybe_find_by_id(new_payment.id).await?.is_some() {
            return Ok(false);
        }

        self.repo.create_in_op(db, new_payment).await?;

        Ok(true)
    }
}
