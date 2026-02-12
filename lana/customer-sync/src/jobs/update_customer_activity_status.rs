use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{
    CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction, GovernanceObject,
};
use core_time_events::CoreTimeEvent;
use governance::GovernanceEvent;
use job::*;
use lana_events::LanaEvent;
use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

#[derive(Default, Clone, Deserialize, Serialize)]
struct UpdateCustomerActivityStatusJobData {
    sequence: EventSequence,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct UpdateCustomerActivityStatusJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> UpdateCustomerActivityStatusJobConfig<Perms, E> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> Clone for UpdateCustomerActivityStatusJobConfig<Perms, E> {
    fn clone(&self) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

pub(crate) struct UpdateCustomerActivityStatusInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    outbox: Outbox<E>,
    customers: Customers<Perms, E>,
}

impl<Perms, E> UpdateCustomerActivityStatusInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    pub(crate) fn new(outbox: &Outbox<E>, customers: &Customers<Perms, E>) -> Self {
        Self {
            outbox: outbox.clone(),
            customers: customers.clone(),
        }
    }
}

const UPDATE_CUSTOMER_ACTIVITY_STATUS: JobType =
    JobType::new("outbox.update-customer-activity-status");

impl<Perms, E> JobInitializer for UpdateCustomerActivityStatusInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    type Config = UpdateCustomerActivityStatusJobConfig<Perms, E>;
    fn job_type(&self) -> JobType {
        UPDATE_CUSTOMER_ACTIVITY_STATUS
    }

    fn init(
        &self,
        _: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdateCustomerActivityStatusJobRunner {
            outbox: self.outbox.clone(),
            customers: self.customers.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub(crate) struct UpdateCustomerActivityStatusJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    outbox: Outbox<E>,
    customers: Customers<Perms, E>,
}

impl<Perms, E> UpdateCustomerActivityStatusJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    #[instrument(name = "customer_sync.update_customer_activity_status.process_message", parent = None, skip(self, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    #[allow(clippy::single_match)]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.as_event() {
            Some(event @ CoreTimeEvent::EndOfDay { closing_time, .. }) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                self.customers
                    .perform_customer_activity_status_update(*closing_time)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait]
impl<Perms, E> JobRunner for UpdateCustomerActivityStatusJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<UpdateCustomerActivityStatusJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %UPDATE_CUSTOMER_ACTIVITY_STATUS,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            self.process_message(message.as_ref()).await?;
                            state.sequence = message.sequence;
                            current_job.update_execution_state(&state).await?;
                        }
                        None => {
                            return Ok(JobCompletion::RescheduleNow);
                        }
                    }
                }
            }
        }
    }
}
