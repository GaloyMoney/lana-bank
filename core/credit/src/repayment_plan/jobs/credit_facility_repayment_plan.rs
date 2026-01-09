use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use futures::StreamExt;
use std::sync::Arc;

use es_entity::clock::ClockHandle;
use job::*;
use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{event::CoreCreditEvent, repayment_plan::*};

#[derive(Serialize, Deserialize)]
pub struct RepaymentPlanProjectionConfig<E> {
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for RepaymentPlanProjectionConfig<E> {
    fn clone(&self) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct RepaymentPlanProjectionInit<E: OutboxEventMarker<CoreCreditEvent>> {
    outbox: Outbox<E>,
    repo: Arc<RepaymentPlanRepo>,
}

impl<E> RepaymentPlanProjectionInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(outbox: &Outbox<E>, repo: Arc<RepaymentPlanRepo>) -> Self {
        Self {
            outbox: outbox.clone(),
            repo,
        }
    }
}

const REPAYMENT_PLAN_PROJECTION: JobType =
    JobType::new("outbox.credit-facility-repayment-plan-projection");
impl<E> JobInitializer for RepaymentPlanProjectionInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Config = RepaymentPlanProjectionConfig<E>;
    fn job_type(&self) -> JobType {
        REPAYMENT_PLAN_PROJECTION
    }

    fn init(
        &self,
        _job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(RepaymentPlanProjectionJobRunner {
            outbox: self.outbox.clone(),
            repo: self.repo.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, Deserialize, Serialize)]
struct RepaymentPlanProjectionJobData {
    sequence: EventSequence,
}

pub struct RepaymentPlanProjectionJobRunner<E: OutboxEventMarker<CoreCreditEvent>> {
    outbox: Outbox<E>,
    repo: Arc<RepaymentPlanRepo>,
}

impl<E> RepaymentPlanProjectionJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(name = "outbox.core_credit.repayment_plan_projection_job.process_message", parent = None, skip(self, message, db, sequence, clock), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        sequence: EventSequence,
        clock: &ClockHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        match message.as_event() {
            Some(event @ FacilityProposalCreated { id, .. })
            | Some(
                event @ FacilityProposalConcluded {
                    id,
                    status: crate::primitives::CreditFacilityProposalStatus::Approved,
                },
            ) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                let facility_id: crate::primitives::CreditFacilityId = (*id).into();
                let mut repayment_plan = self.repo.load(facility_id).await?;
                repayment_plan.process_event(sequence, event, clock.now());
                self.repo
                    .persist_in_tx(db, facility_id, repayment_plan)
                    .await?;
            }
            Some(event @ FacilityActivated { id, .. })
            | Some(event @ FacilityCompleted { id, .. })
            | Some(
                event @ FacilityPaymentAllocated {
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
                event @ PartialLiquidationInitiated {
                    credit_facility_id: id,
                    ..
                },
            )
            | Some(
                event @ PartialLiquidationCompleted {
                    credit_facility_id: id,
                    ..
                },
            ) => {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());

                let mut repayment_plan = self.repo.load(*id).await?;
                repayment_plan.process_event(sequence, event, clock.now());
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

        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %REPAYMENT_PLAN_PROJECTION,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            let mut db = self.repo.begin().await?;
                            self.process_message(&message, &mut db, state.sequence, current_job.clock())
                                .await?;

                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut db, &state)
                                .await?;
                            db.commit().await?;
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
