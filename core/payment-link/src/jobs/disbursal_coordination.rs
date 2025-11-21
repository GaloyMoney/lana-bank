use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use tracing::instrument;

use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings,
};
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use core_credit::CoreCreditEvent;

use crate::{event::CorePaymentLinkEvent, CorePaymentLink};

#[derive(serde::Serialize)]
pub struct DisbursalCoordinationJobConfig<E> {
    _phantom: std::marker::PhantomData<E>,
}

impl<E> DisbursalCoordinationJobConfig<E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> Default for DisbursalCoordinationJobConfig<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E> JobConfig for DisbursalCoordinationJobConfig<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CorePaymentLinkEvent>
        + std::fmt::Debug
        + Send
        + 'static
        + Sync,
{
    type Initializer = DisbursalCoordinationInit<E>;
}

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct DisbursalCoordinationJobData {
    sequence: outbox::EventSequence,
}

pub struct DisbursalCoordinationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CorePaymentLinkEvent>,
{
    payment_links: CorePaymentLink<E>,
    outbox: Outbox<E>,
}

impl<E> DisbursalCoordinationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CorePaymentLinkEvent>
        + std::fmt::Debug
        + Send
        + 'static
        + Sync,
{
    #[instrument(
        name = "core_payment_link.disbursal_coordination_job.process_message",
        parent = None,
        skip(payment_links, message),
        fields(handled = false, event_type = tracing::field::Empty)
    )]
    async fn process_message_static(
        payment_links: &CorePaymentLink<E>,
        message: &Arc<PersistentOutboxEvent<E>>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        E: OutboxEventMarker<CoreCreditEvent>,
    {
        // Extract Credit events from the outbox message
        if let Some(credit_event) =
            PersistentOutboxEvent::as_event::<CoreCreditEvent>(message.as_ref())
        {
            match credit_event {
                // When a facility is activated, ensure a FundingLink exists
                CoreCreditEvent::FacilityActivated {
                    id: credit_facility_id,
                    ..
                } => {
                    // Check if FundingLink already exists for this facility
                    let existing_link = payment_links
                        .find_by_credit_facility(*credit_facility_id)
                        .await?;

                    if existing_link.is_none() {
                        // FundingLink doesn't exist yet
                        // In a full implementation, this would:
                        // 1. Look up the customer_id and deposit_account_id for this facility
                        // 2. Create a FundingLink
                        // 3. Or emit an event indicating manual intervention is needed
                        tracing::warn!(
                            credit_facility_id = %credit_facility_id,
                            "FacilityActivated but no FundingLink exists - manual intervention may be required"
                        );
                    }
                }

                // When a disbursal is settled, verify the FundingLink is active
                CoreCreditEvent::DisbursalSettled {
                    credit_facility_id,
                    amount,
                    ..
                } => {
                    let link = payment_links
                        .find_by_credit_facility(*credit_facility_id)
                        .await?;

                    match link {
                        Some(funding_link) if funding_link.is_active() => {
                            tracing::info!(
                                credit_facility_id = %credit_facility_id,
                                amount = %amount,
                                "Disbursal settled through active FundingLink"
                            );
                        }
                        Some(funding_link) => {
                            tracing::warn!(
                                credit_facility_id = %credit_facility_id,
                                status = ?funding_link.status,
                                "Disbursal settled but FundingLink is not active"
                            );
                        }
                        None => {
                            tracing::error!(
                                credit_facility_id = %credit_facility_id,
                                "Disbursal settled but no FundingLink exists - data inconsistency!"
                            );
                        }
                    }
                }

                _ => {
                    // Other Credit events are not relevant for disbursal coordination
                }
            }
        }
        Ok(())
    }
}

pub struct DisbursalCoordinationInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CorePaymentLinkEvent>,
{
    outbox: Outbox<E>,
    payment_links: CorePaymentLink<E>,
}

impl<E> DisbursalCoordinationInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CorePaymentLinkEvent>,
{
    pub fn new(outbox: &Outbox<E>, payment_links: &CorePaymentLink<E>) -> Self {
        Self {
            outbox: outbox.clone(),
            payment_links: payment_links.clone(),
        }
    }
}

const DISBURSAL_COORDINATION_JOB: JobType =
    JobType::new("outbox.payment-link-disbursal-coordination");

impl<E> JobInitializer for DisbursalCoordinationInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CorePaymentLinkEvent>
        + std::fmt::Debug
        + Send
        + 'static
        + Sync,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        DISBURSAL_COORDINATION_JOB
    }

    #[instrument(
        name = "core_payment_link.disbursal_coordination_job.init",
        skip_all,
        err
    )]
    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(DisbursalCoordinationJobRunner {
            outbox: self.outbox.clone(),
            payment_links: self.payment_links.clone(),
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
impl<E> JobRunner for DisbursalCoordinationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CorePaymentLinkEvent>
        + std::fmt::Debug
        + Send
        + 'static
        + Sync,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<DisbursalCoordinationJobData>()?
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
