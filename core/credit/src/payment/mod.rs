mod entity;
pub mod error;
mod repo;

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
    repo: PaymentRepo,
    authz: Perms,
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
    pub fn new(pool: &sqlx::PgPool, authz: &Perms) -> Self {
        let repo = PaymentRepo::new(pool);

        Self {
            repo,
            authz: authz.clone(),
        }
    }

    pub(super) async fn record_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        credit_facility_id: CreditFacilityId,
        amount: UsdCents,
        audit_info: &audit::AuditInfo,
    ) -> Result<Payment, PaymentError> {
        let new_payment = NewPayment::builder()
            .id(PaymentId::new())
            .amount(amount)
            .credit_facility_id(credit_facility_id)
            .audit_info(audit_info.clone())
            .build()
            .expect("could not build new payment");

        let payment = self.repo.create_in_op(db, new_payment).await?;

        Ok(payment)
    }
}
