mod entity;
pub mod error;
mod repo;

use std::sync::Arc;

use audit::AuditSvc;
use authz::PermissionCheck;

use crate::{CoreCreditAction, CoreCreditObject, primitives::*};

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
}

impl<Perms> Clone for Payments<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            authz: self.authz.clone(),
        }
    }
}

impl<Perms> Payments<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    pub fn new(pool: &sqlx::PgPool, authz: Arc<Perms>) -> Self {
        let repo = PaymentRepo::new(pool);

        Self {
            repo: Arc::new(repo),
            authz,
        }
    }

    /// Attempts to create new Payment entity with
    /// `payment_id`. Returns `true` if the new entity was created
    /// (i. e. `payment_id` was not previously used), otherwise
    /// returns `false`.
    pub(super) async fn record_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        payment_id: PaymentId,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,
    ) -> Result<bool, PaymentError> {
        let new_payment = NewPayment::builder()
            .id(payment_id)
            .amount(amount)
            .credit_facility_id(credit_facility_id)
            .build()
            .expect("could not build new payment");

        if self.repo.maybe_find_by_id(payment_id).await?.is_some() {
            Ok(false)
        } else {
            self.repo.create_in_op(db, new_payment).await?;
            Ok(true)
        }
    }
}
