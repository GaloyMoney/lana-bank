use futures::{FutureExt, StreamExt, select};
use serde::{Deserialize, Serialize};
use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use job::*;
use outbox::{
    EphemeralOutboxEvent, EventSequence, Outbox, OutboxEvent, OutboxEventMarker,
    PersistentOutboxEvent,
};

use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::CorePriceEvent;

use crate::{
    event::CoreCreditEvent, pending_credit_facility::PendingCreditFacilities, primitives::*,
};

#[derive(Serialize, Deserialize)]
pub struct PendingCreditFacilityCollateralizationFromEventsJobConfig<Perms, E> {
    pub _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> JobConfig for PendingCreditFacilityCollateralizationFromEventsJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    type Initializer = PendingCreditFacilityCollateralizationFromEventsInit<Perms, E>;
}

pub struct PendingCreditFacilityCollateralizationFromEventsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    outbox: Outbox<E>,
    pending_credit_facilities: PendingCreditFacilities<Perms, E>,
}

impl<Perms, E> PendingCreditFacilityCollateralizationFromEventsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        pending_credit_facilities: &PendingCreditFacilities<Perms, E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            pending_credit_facilities: pending_credit_facilities.clone(),
        }
    }
}

const PENDING_CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB: JobType =
    JobType::new("outbox.pending-credit-facility-collateralization-from-events");
impl<Perms, E> JobInitializer for PendingCreditFacilityCollateralizationFromEventsInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        PENDING_CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB
    }

    fn init(&self, _job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(
            PendingCreditFacilityCollateralizationFromEventsRunner::<Perms, E> {
                outbox: self.outbox.clone(),
                pending_credit_facility: self.pending_credit_facilities.clone(),
            },
        ))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

// TODO: reproduce 'collateralization_ratio' test from old credit facility

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct PendingCreditFacilityCollateralizationFromEventsData {
    sequence: EventSequence,
}

pub struct PendingCreditFacilityCollateralizationFromEventsRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    outbox: Outbox<E>,
    pending_credit_facility: PendingCreditFacilities<Perms, E>,
}

impl<Perms, E> PendingCreditFacilityCollateralizationFromEventsRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[instrument(name = "core_credit.pending_credit_facility_collateralization_job.process_persistent_message", parent = None, skip(self, message), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty, pending_credit_facility_id = tracing::field::Empty))]
    #[allow(clippy::single_match)]
    async fn process_persistent_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.as_event() {
            Some(
                event @ CoreCreditEvent::FacilityCollateralUpdated {
                    pending_credit_facility_id: id,
                    ..
                },
            ) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());
                Span::current().record("pending_credit_facility_id", tracing::field::display(id));

                self.pending_credit_facility
                    .update_collateralization_from_events(*id)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    #[instrument(name = "core_credit.pending_credit_facility_collateralization_job.process_ephemeral_message", parent = None, skip(self, message), fields(handled = false, event_type = tracing::field::Empty))]
    #[allow(clippy::single_match)]
    async fn process_ephemeral_message(
        &self,
        message: &EphemeralOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.payload.as_event() {
            Some(CorePriceEvent::PriceUpdated { price, .. }) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", tracing::field::display(&message.event_type));

                self.pending_credit_facility
                    .update_collateralization_from_price_event(*price)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl<Perms, E> JobRunner for PendingCreditFacilityCollateralizationFromEventsRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<GovernanceAction> + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<GovernanceObject> + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<PendingCreditFacilityCollateralizationFromEventsData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_all(Some(state.sequence)).await?;

        loop {
            select! {
                event = stream.next().fuse() => {
                    match event {
                        Some(event) => {
                            match event {
                                OutboxEvent::Persistent(e) => {
                                    self.process_persistent_message(&e).await?;
                                    state.sequence = e.sequence;
                                    current_job.update_execution_state(state).await?;
                                }
                                OutboxEvent::Ephemeral(e) => {
                                    self.process_ephemeral_message(e.as_ref()).await?;
                                }
                            }
                        }
                        None => {
                            return Ok(JobCompletion::RescheduleNow);
                        }
                    }
                }
                _ = current_job.shutdown_requested().fuse() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %PENDING_CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
            }
        }
    }
}
