use async_trait::async_trait;
use tracing_macros::record_error_severity;
use futures::StreamExt;
use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{
    CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction, GovernanceObject,
};

use governance::GovernanceEvent;
use lana_events::{LanaEvent, TimeEvent};
use outbox::{Outbox, OutboxEventMarker};

use job::*;

#[derive(serde::Serialize)]
pub struct UpdateCustomerActivityStatusJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> UpdateCustomerActivityStatusJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> Default for UpdateCustomerActivityStatusJobConfig<Perms, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Perms, E> JobConfig for UpdateCustomerActivityStatusJobConfig<Perms, E>
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
        + OutboxEventMarker<TimeEvent>,
{
    type Initializer = UpdateCustomerActivityStatusInit<Perms, E>;
}

pub struct UpdateCustomerActivityStatusInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<TimeEvent>,
{
    customers: Customers<Perms, E>,
    outbox: Outbox<E>,
}

impl<Perms, E> UpdateCustomerActivityStatusInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<TimeEvent>,
{
    pub fn new(customers: &Customers<Perms, E>, outbox: &Outbox<E>) -> Self {
        Self {
            customers: customers.clone(),
            outbox: outbox.clone(),
        }
    }
}

const UPDATE_CUSTOMER_ACTIVITY_STATUS: JobType =
    JobType::new("cron.update-customer-activity-status");

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
        + OutboxEventMarker<TimeEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        UPDATE_CUSTOMER_ACTIVITY_STATUS
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdateCustomerActivityStatusJobRunner {
            customers: self.customers.clone(),
            outbox: self.outbox.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct UpdateCustomerActivityStatusJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<TimeEvent>,
{
    customers: Customers<Perms, E>,
    outbox: Outbox<E>,
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
        + OutboxEventMarker<TimeEvent>,
{
    #[instrument(
        name = "update_customer_activity_status.process_message",
        parent = None,
        skip(self, event),
        fields(event_type = ?event.event_type, handled = false, date = tracing::field::Empty, closing_timestamp = tracing::field::Empty)
    )]
    async fn process_message(
        &self,
        event: &outbox::EphemeralOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(TimeEvent::DailyClosing { date }) = event.payload.as_event() {
            event.inject_trace_parent();
            Span::current().record("date", date.to_string());
            Span::current().record("handled", true);

            // Use the end of the closing day as the reference timestamp
            // This ensures consistent threshold calculations even if the job is restarted
            let closing_timestamp = date
                .and_hms_opt(23, 59, 59)
                .ok_or("Invalid date for closing timestamp")?
                .and_utc();

            Span::current().record("closing_timestamp", closing_timestamp.to_rfc3339());

            self.customers
                .perform_customer_activity_status_update(closing_timestamp)
                .await?;
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
        + OutboxEventMarker<TimeEvent>,
{
    #[record_error_severity]
    #[instrument(name = "update_customer_activity_status.run", skip(self, _current_job))]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut stream = self.outbox.listen_ephemeral().await?;

        while let Some(event) = stream.next().await {
            self.process_message(&event).await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
