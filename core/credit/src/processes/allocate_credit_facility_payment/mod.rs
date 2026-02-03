mod job;

use std::sync::Arc;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_accounting::LedgerTransactionInitiator;
use obix::out::OutboxEventMarker;

use crate::{
    CoreCreditAction, CoreCreditCollectionAction, CoreCreditCollectionEvent,
    CoreCreditCollectionObject, CoreCreditObject, Obligations, Payments, error::CoreCreditError,
    event::CoreCreditEvent, primitives::PaymentId,
};

pub use job::*;

pub struct AllocateCreditFacilityPayment<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    payments: Arc<Payments<Perms, E>>,
    obligations: Arc<Obligations<Perms, E>>,
}

impl<Perms, E> Clone for AllocateCreditFacilityPayment<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    fn clone(&self) -> Self {
        Self {
            payments: self.payments.clone(),
            obligations: self.obligations.clone(),
        }
    }
}

impl<Perms, E> AllocateCreditFacilityPayment<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(payments: Arc<Payments<Perms, E>>, obligations: Arc<Obligations<Perms, E>>) -> Self {
        Self {
            payments,
            obligations,
        }
    }

    pub(super) async fn begin_op(&self) -> Result<es_entity::DbOp<'_>, CoreCreditError> {
        Ok(self
            .obligations
            .begin_op()
            .await
            .map_err(core_credit_collection::CoreCreditCollectionError::from)?)
    }

    #[instrument(
        name = "credit.allocate_credit_facility_payment.execute_in_op",
        skip(self, db)
    )]
    pub async fn execute_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        payment_id: PaymentId,
        initiated_by: LedgerTransactionInitiator,
    ) -> Result<(), CoreCreditError> {
        if let Some(payment) = self
            .payments
            .find_by_id(payment_id)
            .await
            .map_err(core_credit_collection::CoreCreditCollectionError::from)?
        {
            self.obligations
                .allocate_payment_in_op(db, payment.into(), initiated_by)
                .await
                .map_err(core_credit_collection::CoreCreditCollectionError::from)?;
        }
        Ok(())
    }
}
