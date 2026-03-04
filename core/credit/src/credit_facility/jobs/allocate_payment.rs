use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{Span, instrument};

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use core_credit_collection::CoreCreditCollection;
use job::*;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};
use tracing_macros::observe_error;

use crate::{
    CoreCreditAction, CoreCreditCollectionAction, CoreCreditCollectionEvent,
    CoreCreditCollectionObject, CoreCreditEvent, CoreCreditObject, primitives::PaymentId,
};

pub const ALLOCATE_CREDIT_FACILITY_PAYMENT: JobType =
    JobType::new("outbox.allocate-credit-facility-payment");

pub const ALLOCATE_CREDIT_FACILITY_PAYMENT_COMMAND: JobType =
    JobType::new("command.credit.allocate-credit-facility-payment");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AllocatePaymentConfig {
    pub payment_id: PaymentId,
}

pub struct AllocatePaymentJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    collections: Arc<CoreCreditCollection<Perms, E>>,
}

impl<Perms, E> AllocatePaymentJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(collections: Arc<CoreCreditCollection<Perms, E>>) -> Self {
        Self { collections }
    }
}

impl<Perms, E> JobInitializer for AllocatePaymentJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    type Config = AllocatePaymentConfig;

    fn job_type(&self) -> JobType {
        ALLOCATE_CREDIT_FACILITY_PAYMENT_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(AllocatePaymentJobRunner {
            config: job.config()?,
            collections: self.collections.clone(),
        }))
    }
}

struct AllocatePaymentJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    config: AllocatePaymentConfig,
    collections: Arc<CoreCreditCollection<Perms, E>>,
}

#[async_trait]
impl<Perms, E> JobRunner for AllocatePaymentJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    #[observe_error(allow_single_error_alert)]
    #[tracing::instrument(
        name = "credit.allocate_credit_facility_payment.process_command",
        skip_all
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        if let Some(payment) = self
            .collections
            .payments()
            .find_by_id_in_op(&mut op, self.config.payment_id)
            .await?
        {
            self.collections
                .obligations()
                .allocate_payment_in_op(
                    &mut op,
                    payment.into(),
                    &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(
                        crate::primitives::CREDIT_FACILITY_PAYMENT_ALLOCATION,
                    ),
                )
                .await?;
        }

        Ok(JobCompletion::CompleteWithOp(op))
    }
}

pub struct AllocateCreditFacilityPaymentHandler {
    allocate_payment: JobSpawner<AllocatePaymentConfig>,
}

impl AllocateCreditFacilityPaymentHandler {
    pub fn new(allocate_payment: JobSpawner<AllocatePaymentConfig>) -> Self {
        Self { allocate_payment }
    }
}

impl<E> OutboxEventHandler<E> for AllocateCreditFacilityPaymentHandler
where
    E: OutboxEventMarker<CoreCreditCollectionEvent>,
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

            self.allocate_payment
                .spawn_with_queue_id_in_op(
                    op,
                    JobId::new(),
                    AllocatePaymentConfig {
                        payment_id: entity.id,
                    },
                    entity.id.to_string(),
                )
                .await?;
        }
        Ok(())
    }
}
