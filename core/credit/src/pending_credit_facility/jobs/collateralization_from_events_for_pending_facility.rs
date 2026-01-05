use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};
use tracing_macros::record_error_severity;

use std::sync::Arc;

use governance::GovernanceEvent;
use job_new::*;
use obix::EventSequence;
use obix::out::{
    EphemeralOutboxEvent, Outbox, OutboxEvent, OutboxEventMarker, PersistentOutboxEvent,
};

use core_custody::CoreCustodyEvent;
use core_price::{CorePriceEvent, Price};

use crate::{
    event::CoreCreditEvent,
    ledger::*,
    pending_credit_facility::{
        PendingCreditFacilitiesByCollateralizationRatioCursor, PendingCreditFacility,
        PendingCreditFacilityError, PendingCreditFacilityRepo,
    },
    primitives::*,
};

#[derive(Serialize, Deserialize)]
pub struct PendingCreditFacilityCollateralizationFromEventsJobConfig<E> {
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for PendingCreditFacilityCollateralizationFromEventsJobConfig<E> {
    fn clone(&self) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct PendingCreditFacilityCollateralizationFromEventsInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    outbox: Outbox<E>,
    repo: Arc<PendingCreditFacilityRepo<E>>,
    price: Arc<Price>,
    ledger: Arc<CreditLedger>,
}

impl<E> PendingCreditFacilityCollateralizationFromEventsInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        repo: Arc<PendingCreditFacilityRepo<E>>,
        price: Arc<Price>,
        ledger: Arc<CreditLedger>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            repo,
            price,
            ledger,
        }
    }
}

const PENDING_CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB: JobType =
    JobType::new("outbox.pending-credit-facility-collateralization-from-events");
impl<E> JobInitializer for PendingCreditFacilityCollateralizationFromEventsInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    type Config = PendingCreditFacilityCollateralizationFromEventsJobConfig<E>;
    fn job_type(&self) -> JobType
    where
        Self: Sized,
    {
        PENDING_CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB
    }

    fn init(
        &self,
        _job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(
            PendingCreditFacilityCollateralizationFromEventsRunner::<E> {
                outbox: self.outbox.clone(),
                repo: self.repo.clone(),
                price: self.price.clone(),
                ledger: self.ledger.clone(),
            },
        ))
    }

    fn retry_on_error_settings(&self) -> RetrySettings
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

pub struct PendingCreditFacilityCollateralizationFromEventsRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    outbox: Outbox<E>,
    repo: Arc<PendingCreditFacilityRepo<E>>,
    price: Arc<Price>,
    ledger: Arc<CreditLedger>,
}

impl<E> PendingCreditFacilityCollateralizationFromEventsRunner<E>
where
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

                self.update_collateralization_from_events(*id).await?;
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

                self.update_collateralization_from_price_event(*price)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.pending_credit_facility.update_collateralization_from_events",
        skip(self)
    )]
    #[es_entity::retry_on_concurrent_modification(any_error = true)]
    pub(super) async fn update_collateralization_from_events(
        &self,
        id: PendingCreditFacilityId,
    ) -> Result<PendingCreditFacility, PendingCreditFacilityError> {
        let mut op = self.repo.begin_op().await?;
        let mut pending_facility = self.repo.find_by_id_in_op(&mut op, id).await?;

        tracing::Span::current().record(
            "pending_credit_facility_id",
            pending_facility.id.to_string(),
        );

        let balances = self
            .ledger
            .get_pending_credit_facility_balance(pending_facility.account_ids)
            .await?;

        let price = self.price.usd_cents_per_btc().await;

        if pending_facility
            .update_collateralization(price, balances)
            .did_execute()
        {
            self.repo
                .update_in_op(&mut op, &mut pending_facility)
                .await?;

            op.commit().await?;
        }
        Ok(pending_facility)
    }

    #[record_error_severity]
    #[instrument(
        name = "credit.credit_facility.update_collateralization_from_price_event",
        skip(self)
    )]
    pub(super) async fn update_collateralization_from_price_event(
        &self,
        price: PriceOfOneBTC,
    ) -> Result<(), PendingCreditFacilityError> {
        let mut has_next_page = true;
        let mut after: Option<PendingCreditFacilitiesByCollateralizationRatioCursor> = None;
        while has_next_page {
            let mut pending_credit_facilities = self
                .repo
                .list_by_collateralization_ratio(
                    es_entity::PaginatedQueryArgs::<
                        PendingCreditFacilitiesByCollateralizationRatioCursor,
                    > {
                        first: 10,
                        after,
                    },
                    Default::default(),
                )
                .await?;
            (after, has_next_page) = (
                pending_credit_facilities.end_cursor,
                pending_credit_facilities.has_next_page,
            );
            let mut op = self.repo.begin_op().await?;

            let mut at_least_one = false;

            for pending_facility in pending_credit_facilities.entities.iter_mut() {
                tracing::Span::current().record(
                    "pending_credit_facility_id",
                    pending_facility.id.to_string(),
                );

                if pending_facility.status() == PendingCreditFacilityStatus::Completed {
                    continue;
                }
                let balances = self
                    .ledger
                    .get_pending_credit_facility_balance(pending_facility.account_ids)
                    .await?;
                if pending_facility
                    .update_collateralization(price, balances)
                    .did_execute()
                {
                    self.repo.update_in_op(&mut op, pending_facility).await?;
                    at_least_one = true;
                }
            }

            if at_least_one {
                op.commit().await?;
            } else {
                break;
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl<E> JobRunner for PendingCreditFacilityCollateralizationFromEventsRunner<E>
where
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
        let mut stream = self.outbox.listen_all(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %PENDING_CREDIT_FACILITY_COLLATERALIZATION_FROM_EVENTS_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                event = stream.next() => {
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
            }
        }
    }
}
