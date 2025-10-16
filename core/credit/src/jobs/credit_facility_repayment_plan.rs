use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{Span, instrument};

use job::*;
use outbox::{EventSequence, Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{event::CoreCreditEvent, repayment_plan::*};

#[derive(Default, Clone, Deserialize, Serialize)]
struct RepaymentPlanProjectionJobData {
    sequence: EventSequence,
}

pub struct RepaymentPlanProjectionJobRunner<E: OutboxEventMarker<CoreCreditEvent>> {
    outbox: Outbox<E>,
    repo: RepaymentPlanRepo,
}

impl<E> RepaymentPlanProjectionJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(name = "outbox.core_credit.repayment_plan_projection_job.process_message", parent = None, skip(self, message, db, sequence), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        sequence: EventSequence,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        match message.as_event() {
            Some(event @ FacilityProposalCreated { id, .. })
            | Some(event @ FacilityProposalApproved { id, .. }) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                let facility_id: crate::primitives::CreditFacilityId = (*id).into();
                let mut repayment_plan = self.repo.load(facility_id).await?;
                repayment_plan.process_event(sequence, event);
                self.repo
                    .persist_in_tx(db, facility_id, repayment_plan)
                    .await?;
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

                let mut repayment_plan = self.repo.load(*id).await?;
                repayment_plan.process_event(sequence, event);
                self.repo.persist_in_tx(db, *id, repayment_plan).await?;
            }
            _ => {}
        }
        Ok(())
    }
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
        let mut state = current_job
            .execution_state::<RepaymentPlanProjectionJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            let mut db = self.repo.begin().await?;
            self.process_message(&message, &mut db, state.sequence)
                .await?;

            state.sequence = message.sequence;
            current_job
                .update_execution_state_in_op(&mut db, &state)
                .await?;

            db.commit().await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

pub struct RepaymentPlanProjectionInit<E: OutboxEventMarker<CoreCreditEvent>> {
    outbox: Outbox<E>,
    repo: RepaymentPlanRepo,
}

impl<E> RepaymentPlanProjectionInit<E>
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
    JobType::new("outbox.credit-facility-repayment-plan-projection");
impl<E> JobInitializer for RepaymentPlanProjectionInit<E>
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

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RepaymentPlanProjectionConfig<E> {
    pub _phantom: std::marker::PhantomData<E>,
}
impl<E> JobConfig for RepaymentPlanProjectionConfig<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Initializer = RepaymentPlanProjectionInit<E>;
}
