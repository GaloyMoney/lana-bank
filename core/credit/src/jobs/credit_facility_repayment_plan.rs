use futures::StreamExt;
use serde::{Deserialize, Serialize};

use job::*;
use outbox::{EventSequence, Outbox, OutboxEventMarker};

use crate::{event::CoreCreditEvent, repayment_plan::*};

#[derive(Default, Clone, Deserialize, Serialize)]
struct RepaymentPlanProjectionJobData {
    sequence: EventSequence,
}

pub struct RepaymentPlanProjectionJobRunner<E: OutboxEventMarker<CoreCreditEvent>> {
    outbox: Outbox<E>,
    repo: RepaymentPlanRepo,
}

#[async_trait::async_trait]
impl<E> JobRunner for RepaymentPlanProjectionJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        let mut state = current_job
            .execution_state::<RepaymentPlanProjectionJobData>()?
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
                    } => *id,
                };

                let mut db = self.repo.begin().await?;

                let mut repayment_plan = self.repo.load(id).await?;
                repayment_plan.process_event(state.sequence, event);
                self.repo.persist_in_tx(&mut db, id, repayment_plan).await?;

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

pub struct RepaymentPlanProjectionInitializer<E: OutboxEventMarker<CoreCreditEvent>> {
    outbox: Outbox<E>,
    repo: RepaymentPlanRepo,
}

impl<E> RepaymentPlanProjectionInitializer<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(outbox: &Outbox<E>, repo: &RepaymentPlanRepo) -> Self {
        Self {
            outbox: outbox.clone(),
            repo: repo.clone(),
        }
    }
}

const REPAYMENT_PLAN_PROJECTION: JobType =
    JobType::new("credit-facility-repayment-plan-projection");
impl<E> JobInitializer for RepaymentPlanProjectionInitializer<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        REPAYMENT_PLAN_PROJECTION
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(RepaymentPlanProjectionJobRunner {
            outbox: self.outbox.clone(),
            repo: self.repo.clone(),
        }))
    }
}

#[derive(Serialize, Deserialize)]
pub struct RepaymentPlanProjectionConfig<E> {
    pub _phantom: std::marker::PhantomData<E>,
}
impl<E> JobConfig for RepaymentPlanProjectionConfig<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Initializer = RepaymentPlanProjectionInitializer<E>;
}
