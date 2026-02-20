mod job;

use audit::SystemSubject;
use std::sync::Arc;

use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_credit_collection::{CollectionLedgerOps, CoreCreditCollection};
use obix::out::OutboxEventMarker;

use crate::{
    CoreCreditAction, CoreCreditCollectionAction, CoreCreditCollectionEvent,
    CoreCreditCollectionObject, CoreCreditEvent, CoreCreditObject, error::CoreCreditError,
    primitives::PaymentId,
};

pub use job::*;

pub struct AllocateCreditFacilityPayment<Perms, E, ColL>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
    ColL: CollectionLedgerOps,
{
    collections: Arc<CoreCreditCollection<Perms, E, ColL>>,
}

impl<Perms, E, ColL> Clone for AllocateCreditFacilityPayment<Perms, E, ColL>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
    ColL: CollectionLedgerOps,
{
    fn clone(&self) -> Self {
        Self {
            collections: self.collections.clone(),
        }
    }
}

impl<Perms, E, ColL> AllocateCreditFacilityPayment<Perms, E, ColL>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
    ColL: CollectionLedgerOps,
{
    pub fn new(collections: Arc<CoreCreditCollection<Perms, E, ColL>>) -> Self {
        Self { collections }
    }

    #[instrument(
        name = "credit.allocate_credit_facility_payment.execute_in_op",
        skip(self, db)
    )]
    pub async fn execute_in_op(
        &self,
        db: &mut es_entity::DbOp<'_>,
        payment_id: PaymentId,
        initiated_by: &(impl SystemSubject + Send + Sync),
    ) -> Result<(), CoreCreditError> {
        if let Some(payment) = self.collections.payments().find_by_id(payment_id).await? {
            self.collections
                .obligations()
                .allocate_payment_in_op(db, payment.into(), initiated_by)
                .await?;
        }
        Ok(())
    }
}
