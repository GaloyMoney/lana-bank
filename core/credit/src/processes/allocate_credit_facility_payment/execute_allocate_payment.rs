use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditAction, CoreCreditCollectionAction, CoreCreditCollectionEvent,
    CoreCreditCollectionObject, CoreCreditEvent, CoreCreditObject, primitives::PaymentId,
};

use super::AllocateCreditFacilityPayment;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteAllocatePaymentConfig {
    pub payment_id: PaymentId,
    pub trace_context: tracing_utils::persistence::SerializableTraceContext,
}

pub const EXECUTE_ALLOCATE_PAYMENT_COMMAND: JobType =
    JobType::new("command.credit.execute-allocate-payment");

pub struct ExecuteAllocatePaymentJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    process: AllocateCreditFacilityPayment<Perms, E>,
}

impl<Perms, E> ExecuteAllocatePaymentJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    pub fn new(process: AllocateCreditFacilityPayment<Perms, E>) -> Self {
        Self { process }
    }
}

impl<Perms, E> JobInitializer for ExecuteAllocatePaymentJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    type Config = ExecuteAllocatePaymentConfig;

    fn job_type(&self) -> JobType {
        EXECUTE_ALLOCATE_PAYMENT_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ExecuteAllocatePaymentJobRunner {
            config: job.config()?,
            process: self.process.clone(),
        }))
    }
}

pub struct ExecuteAllocatePaymentJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    config: ExecuteAllocatePaymentConfig,
    process: AllocateCreditFacilityPayment<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for ExecuteAllocatePaymentJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "core_credit.execute_allocate_payment_job.process_command",
        skip(self, current_job),
        fields(payment_id = %self.config.payment_id),
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        tracing_utils::persistence::set_parent(&self.config.trace_context);
        let mut op = current_job.begin_op().await?;
        self.process
            .execute_in_op(
                &mut op,
                self.config.payment_id,
                &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(
                    crate::primitives::CREDIT_FACILITY_PAYMENT_ALLOCATION,
                ),
            )
            .await?;

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
