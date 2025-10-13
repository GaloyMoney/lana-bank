use async_trait::async_trait;
use futures::StreamExt;
use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{CoreCreditAction, CoreCreditEvent, CoreCreditObject};

use super::ApproveDisbursal;

#[derive(serde::Serialize)]
pub(crate) struct DisbursalApprovalJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> DisbursalApprovalJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<Perms, E> Default for DisbursalApprovalJobConfig<Perms, E> {
    fn default() -> Self {
        Self::new()
    }
}
impl<Perms, E> JobConfig for DisbursalApprovalJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    type Initializer = DisbursalApprovalInit<Perms, E>;
}

pub(crate) struct DisbursalApprovalInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    outbox: Outbox<E>,
    process: ApproveDisbursal<Perms, E>,
}

impl<Perms, E> DisbursalApprovalInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(outbox: &Outbox<E>, process: &ApproveDisbursal<Perms, E>) -> Self {
        Self {
            process: process.clone(),
            outbox: outbox.clone(),
        }
    }
}

const DISBURSAL_APPROVE_JOB: JobType = JobType::new("disbursal-approval");
impl<Perms, E> JobInitializer for DisbursalApprovalInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        DISBURSAL_APPROVE_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(DisbursalApprovalJobRunner {
            outbox: self.outbox.clone(),
            process: self.process.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct DisbursalApprovalJobData {
    sequence: outbox::EventSequence,
}

pub struct DisbursalApprovalJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    outbox: Outbox<E>,
    process: ApproveDisbursal<Perms, E>,
}

impl<Perms, E> DisbursalApprovalJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(name = "core_credit.disbursal_approval_job.process_msg", parent = None, skip(self, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty, process_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.as_event() {
            Some(
                event @ GovernanceEvent::ApprovalProcessConcluded {
                    id,
                    approved,
                    process_type,
                    ..
                },
            ) if process_type == &super::APPROVE_DISBURSAL_PROCESS => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());
                Span::current().record("process_type", process_type.to_string());

                self.process.execute(*id, *approved).await?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait]
impl<Perms, E> JobRunner for DisbursalApprovalJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject>,
    E: OutboxEventMarker<GovernanceEvent> + OutboxEventMarker<CoreCreditEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<DisbursalApprovalJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            self.process_message(message.as_ref()).await?;
            state.sequence = message.sequence;
            current_job.update_execution_state(&state).await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
