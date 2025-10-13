use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{Span, instrument};

use job::*;
use outbox::{EventSequence, Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{event::CoreCreditEvent, history::*, primitives::CreditFacilityId};

#[derive(Default, Clone, Deserialize, Serialize)]
struct HistoryProjectionJobData {
    sequence: EventSequence,
}

pub struct HistoryProjectionJobRunner<E: OutboxEventMarker<CoreCreditEvent>> {
    outbox: Outbox<E>,
    repo: HistoryRepo,
}

impl<E> HistoryProjectionJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(name = "core_credit.history_projection_job.process_message", parent = None, skip(self, message, current_job, state), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    #[allow(clippy::single_match)]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
        current_job: &mut CurrentJob,
        state: &mut HistoryProjectionJobData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        match message.as_event() {
            Some(event @ FacilityProposalCreated { id, .. })
            | Some(event @ FacilityProposalApproved { id, .. }) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                let facility_id: CreditFacilityId = (*id).into();

                let mut db = self.repo.begin().await?;
                let mut history = self.repo.load(facility_id).await?;
                history.process_event(event);
                self.repo
                    .persist_in_tx(&mut db, facility_id, history)
                    .await?;

                state.sequence = message.sequence;
                current_job
                    .update_execution_state_in_op(&mut db, state)
                    .await?;
                db.commit().await?;
            }
            Some(event @ FacilityActivated { id, .. })
            | Some(event @ FacilityCompleted { id, .. })
            | Some(
                event @ FacilityRepaymentRecorded {
                    credit_facility_id: id,
                    ..
                },
            )
            | Some(
                event @ FacilityCollateralUpdated {
                    credit_facility_id: id,
                    ..
                },
            )
            | Some(event @ FacilityCollateralizationChanged { id, .. })
            | Some(
                event @ DisbursalSettled {
                    credit_facility_id: id,
                    ..
                },
            )
            | Some(
                event @ AccrualPosted {
                    credit_facility_id: id,
                    ..
                },
            )
            | Some(
                event @ ObligationCreated {
                    credit_facility_id: id,
                    ..
                },
            )
            | Some(
                event @ ObligationDue {
                    credit_facility_id: id,
                    ..
                },
            )
            | Some(
                event @ ObligationOverdue {
                    credit_facility_id: id,
                    ..
                },
            )
            | Some(
                event @ ObligationDefaulted {
                    credit_facility_id: id,
                    ..
                },
            )
            | Some(
                event @ ObligationCompleted {
                    credit_facility_id: id,
                    ..
                },
            )
            | Some(
                event @ LiquidationProcessStarted {
                    credit_facility_id: id,
                    ..
                },
            )
            | Some(
                event @ LiquidationProcessConcluded {
                    credit_facility_id: id,
                    ..
                },
            ) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                let mut db = self.repo.begin().await?;
                let mut history = self.repo.load(*id).await?;
                history.process_event(event);
                self.repo.persist_in_tx(&mut db, *id, history).await?;

                state.sequence = message.sequence;
                current_job
                    .update_execution_state_in_op(&mut db, state)
                    .await?;
                db.commit().await?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl<E> JobRunner for HistoryProjectionJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<HistoryProjectionJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            self.process_message(message.as_ref(), &mut current_job, &mut state)
                .await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

pub struct HistoryProjectionInit<E: OutboxEventMarker<CoreCreditEvent>> {
    outbox: Outbox<E>,
    repo: HistoryRepo,
}

impl<E> HistoryProjectionInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(outbox: &Outbox<E>, repo: &HistoryRepo) -> Self {
        Self {
            outbox: outbox.clone(),
            repo: repo.clone(),
        }
    }
}

const HISTORY_PROJECTION: JobType = JobType::new("credit-facility-history-projection");
impl<E> JobInitializer for HistoryProjectionInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        HISTORY_PROJECTION
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(HistoryProjectionJobRunner {
            outbox: self.outbox.clone(),
            repo: self.repo.clone(),
        }))
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct HistoryProjectionConfig<E> {
    pub _phantom: std::marker::PhantomData<E>,
}
impl<E> JobConfig for HistoryProjectionConfig<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Initializer = HistoryProjectionInit<E>;
}
