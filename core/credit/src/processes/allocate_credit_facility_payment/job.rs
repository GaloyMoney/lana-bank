use tracing::{Span, instrument};

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;

use crate::{
    CoreCreditAction, CoreCreditCollectionAction, CoreCreditCollectionEvent,
    CoreCreditCollectionObject, CoreCreditEvent, CoreCreditObject,
};

use core_credit_collection::CollectionLedgerOps;

use super::AllocateCreditFacilityPayment;

pub const ALLOCATE_CREDIT_FACILITY_PAYMENT: JobType =
    JobType::new("outbox.allocate-credit-facility-payment");

pub struct AllocateCreditFacilityPaymentHandler<Perms, E, ColL>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
    ColL: CollectionLedgerOps,
{
    process: AllocateCreditFacilityPayment<Perms, E, ColL>,
}

impl<Perms, E, ColL> AllocateCreditFacilityPaymentHandler<Perms, E, ColL>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
    ColL: CollectionLedgerOps,
{
    pub fn new(process: &AllocateCreditFacilityPayment<Perms, E, ColL>) -> Self {
        Self {
            process: process.clone(),
        }
    }
}

impl<Perms, E, ColL> OutboxEventHandler<E> for AllocateCreditFacilityPaymentHandler<Perms, E, ColL>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
    ColL: CollectionLedgerOps,
{
    #[instrument(name = "core_credit.allocate_credit_facility_payment_job.process_message_in_op", parent = None, skip(self, op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty, credit_facility_id = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut es_entity::DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use CoreCreditCollectionEvent::*;

        if let Some(e @ PaymentCreated { entity }) = event.as_event() {
            event.inject_trace_parent();
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());
            Span::current().record(
                "credit_facility_id",
                tracing::field::display(&entity.beneficiary_id),
            );

            self.process
                .execute_in_op(
                    op,
                    entity.id,
                    &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(
                        crate::primitives::CREDIT_FACILITY_PAYMENT_ALLOCATION,
                    ),
                )
                .await?;
        }
        Ok(())
    }
}
