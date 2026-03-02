use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use core_credit_collection::CoreCreditCollection;
use job::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditAction, CoreCreditCollectionAction, CoreCreditCollectionEvent,
    CoreCreditCollectionObject, CoreCreditEvent, CoreCreditObject, primitives::PaymentId,
};

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
    #[record_error_severity]
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
