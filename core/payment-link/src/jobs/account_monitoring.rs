use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use tracing::{instrument, Span};

use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings,
};
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use core_deposit::CoreDepositEvent;

use crate::{event::CorePaymentLinkEvent, primitives::BrokenReason, CorePaymentLink};

#[derive(serde::Serialize)]
pub struct AccountMonitoringJobConfig<E> {
    _phantom: std::marker::PhantomData<E>,
}

impl<E> AccountMonitoringJobConfig<E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> Default for AccountMonitoringJobConfig<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E> JobConfig for AccountMonitoringJobConfig<E>
where
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CorePaymentLinkEvent>
        + Send
        + 'static
        + Sync,
{
    type Initializer = AccountMonitoringInit<E>;
}

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct AccountMonitoringJobData {
    sequence: outbox::EventSequence,
}

pub struct AccountMonitoringJobRunner<E>
where
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CorePaymentLinkEvent>
        + Send
        + 'static
        + Sync,
{
    payment_links: CorePaymentLink<E>,
    outbox: Outbox<E>,
}

impl<E> AccountMonitoringJobRunner<E>
where
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CorePaymentLinkEvent>
        + Send
        + 'static
        + Sync,
{
    #[instrument(
        name = "core_payment_link.account_monitoring_job.process_message",
        parent = None,
        skip(payment_links, message),
        fields(handled = false, event_type = tracing::field::Empty)
    )]
    async fn process_message_static(
        payment_links: &CorePaymentLink<E>,
        message: &Arc<PersistentOutboxEvent<E>>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        E: OutboxEventMarker<CoreDepositEvent>,
    {
        // Use the inherent method with explicit type parameter
        if let Some(deposit_event) =
            PersistentOutboxEvent::as_event::<CoreDepositEvent>(message.as_ref())
        {
            if let CoreDepositEvent::DepositAccountFrozen { id, .. } = deposit_event {
                payment_links
                    .break_links_by_deposit_account(*id, BrokenReason::AccountFrozen)
                    .await?;
            }
        }
        Ok(())
    }

    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.as_event() {
            Some(event @ CoreDepositEvent::DepositAccountFrozen { id, .. }) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                self.payment_links
                    .break_links_by_deposit_account(*id, BrokenReason::AccountFrozen)
                    .await?;
            }
            _ => {}
        }

        Ok(())
    }
}

pub struct AccountMonitoringInit<E>
where
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CorePaymentLinkEvent>
        + Send
        + 'static
        + Sync,
{
    outbox: Outbox<E>,
    payment_links: CorePaymentLink<E>,
}

impl<E> AccountMonitoringInit<E>
where
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CorePaymentLinkEvent>
        + Send
        + 'static
        + Sync,
{
    pub fn new(outbox: &Outbox<E>, payment_links: &CorePaymentLink<E>) -> Self {
        Self {
            outbox: outbox.clone(),
            payment_links: payment_links.clone(),
        }
    }
}

const ACCOUNT_MONITORING_JOB: JobType = JobType::new("outbox.payment-link-account-monitoring");

impl<E> JobInitializer for AccountMonitoringInit<E>
where
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CorePaymentLinkEvent>
        + Send
        + 'static
        + Sync,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        ACCOUNT_MONITORING_JOB
    }

    #[instrument(name = "core_payment_link.account_monitoring_job.init", skip_all, err)]
    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(AccountMonitoringJobRunner {
            payment_links: self.payment_links.clone(),
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

#[async_trait]
impl<E> JobRunner for AccountMonitoringJobRunner<E>
where
    E: OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<CorePaymentLinkEvent>
        + Send
        + 'static
        + Sync,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<AccountMonitoringJobData>()?
            .unwrap_or_default();

        let outbox = self.outbox.clone();
        let payment_links = self.payment_links.clone();

        let mut stream = outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            let _ = Self::process_message_static(&payment_links, &message).await?;
            state.sequence = message.sequence;
            current_job.update_execution_state(&state).await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
