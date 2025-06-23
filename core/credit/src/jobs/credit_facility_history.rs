use futures::StreamExt;
use serde::{Deserialize, Serialize};

use job::*;
use outbox::{EventSequence, Outbox, OutboxEventMarker};

use crate::{event::CoreCreditEvent, history::*};

#[derive(Default, Clone, Deserialize, Serialize)]
struct HistoryProjectionJobData {
    sequence: EventSequence,
}

pub struct HistoryProjectionJobRunner<E: OutboxEventMarker<CoreCreditEvent>> {
    outbox: Outbox<E>,
    repo: HistoryRepo,
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
        use CoreCreditEvent::*;

        let mut state = current_job
            .execution_state::<HistoryProjectionJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            if let Some(event) = &message.payload {
                let event = if let Some(event) = event.as_event() {
                    event
                } else {
                    continue;
                };

                let id = match event {
                    FacilityCreated { id, .. }
                    | FacilityApproved { id }
                    | FacilityActivated { id, .. }
                    | FacilityCompleted { id, .. }
                    | FacilityRepaymentRecorded {
                        credit_facility_id: id,
                        ..
                    }
                    | FacilityCollateralUpdated {
                        credit_facility_id: id,
                        ..
                    }
                    | FacilityCollateralizationChanged { id, .. }
                    | DisbursalSettled {
                        credit_facility_id: id,
                        ..
                    }
                    | AccrualPosted {
                        credit_facility_id: id,
                        ..
                    }
                    | ObligationCreated {
                        credit_facility_id: id,
                        ..
                    }
                    | ObligationDue {
                        credit_facility_id: id,
                        ..
                    }
                    | ObligationOverdue {
                        credit_facility_id: id,
                        ..
                    }
                    | ObligationDefaulted {
                        credit_facility_id: id,
                        ..
                    }
                    | ObligationCompleted {
                        credit_facility_id: id,
                        ..
                    }
                    | LiquidationProcessStarted {
                        credit_facility_id: id,
                        ..
                    }
                    | LiquidationProcessConcluded {
                        credit_facility_id: id,
                        ..
                    } => *id,
                };

                let mut db = self.repo.begin().await?;

                let mut history = self.repo.load(id).await?;
                history.process_event(event);
                self.repo.persist_in_tx(&mut db, id, history).await?;

                state.sequence = message.sequence;
                current_job
                    .update_execution_state_in_tx(&mut db, &state)
                    .await?;

                db.commit().await?;
            }
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
pub struct HistoryProjectionConfig<E> {
    pub _phantom: std::marker::PhantomData<E>,
}
impl<E> JobConfig for HistoryProjectionConfig<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Initializer = HistoryProjectionInit<E>;
}
