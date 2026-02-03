use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use futures::StreamExt;
use std::sync::Arc;

use es_entity::clock::ClockHandle;
use job::*;
use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{CoreCreditCollectionEvent, event::CoreCreditEvent, repayment_plan::*};

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

pub struct RepaymentPlanProjectionInit<
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
> {
    outbox: Outbox<E>,
    repo: Arc<RepaymentPlanRepo>,
}

impl<E> RepaymentPlanProjectionInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
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
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
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

pub struct RepaymentPlanProjectionJobRunner<
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
> {
    outbox: Outbox<E>,
    repo: Arc<RepaymentPlanRepo>,
}

impl<E> RepaymentPlanProjectionJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
{
    #[instrument(name = "outbox.core_credit.repayment_plan_projection_job.process_message", parent = None, skip(self, message, db, sequence, clock), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        message: &PersistentOutboxEvent<E>,
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

        if let Some(collection_event) = message.as_event::<CoreCreditCollectionEvent>() {
            let credit_event = match collection_event {
                CoreCreditCollectionEvent::PaymentAllocated {
                    beneficiary_id,
                    obligation_id,
                    obligation_type,
                    allocation_id,
                    amount,
                    recorded_at,
                    effective,
                } => Some((
                    CoreCreditEvent::FacilityPaymentAllocated {
                        credit_facility_id: (*beneficiary_id).into(),
                        obligation_id: *obligation_id,
                        obligation_type: *obligation_type,
                        allocation_id: *allocation_id,
                        amount: *amount,
                        recorded_at: *recorded_at,
                        effective: *effective,
                    },
                    crate::primitives::CreditFacilityId::from(*beneficiary_id),
                )),
                CoreCreditCollectionEvent::ObligationCreated {
                    id,
                    obligation_type,
                    beneficiary_id,
                    amount,
                    due_at,
                    overdue_at,
                    defaulted_at,
                    recorded_at,
                    effective,
                } => Some((
                    CoreCreditEvent::ObligationCreated {
                        id: *id,
                        obligation_type: *obligation_type,
                        credit_facility_id: (*beneficiary_id).into(),
                        amount: *amount,
                        due_at: *due_at,
                        overdue_at: *overdue_at,
                        defaulted_at: *defaulted_at,
                        recorded_at: *recorded_at,
                        effective: *effective,
                    },
                    crate::primitives::CreditFacilityId::from(*beneficiary_id),
                )),
                CoreCreditCollectionEvent::ObligationDue {
                    id,
                    beneficiary_id,
                    obligation_type,
                    amount,
                } => Some((
                    CoreCreditEvent::ObligationDue {
                        id: *id,
                        credit_facility_id: (*beneficiary_id).into(),
                        obligation_type: *obligation_type,
                        amount: *amount,
                    },
                    crate::primitives::CreditFacilityId::from(*beneficiary_id),
                )),
                CoreCreditCollectionEvent::ObligationOverdue {
                    id,
                    beneficiary_id,
                    amount,
                } => Some((
                    CoreCreditEvent::ObligationOverdue {
                        id: *id,
                        credit_facility_id: (*beneficiary_id).into(),
                        amount: *amount,
                    },
                    crate::primitives::CreditFacilityId::from(*beneficiary_id),
                )),
                CoreCreditCollectionEvent::ObligationDefaulted {
                    id,
                    beneficiary_id,
                    amount,
                } => Some((
                    CoreCreditEvent::ObligationDefaulted {
                        id: *id,
                        credit_facility_id: (*beneficiary_id).into(),
                        amount: *amount,
                    },
                    crate::primitives::CreditFacilityId::from(*beneficiary_id),
                )),
                CoreCreditCollectionEvent::ObligationCompleted { id, beneficiary_id } => Some((
                    CoreCreditEvent::ObligationCompleted {
                        id: *id,
                        credit_facility_id: (*beneficiary_id).into(),
                    },
                    crate::primitives::CreditFacilityId::from(*beneficiary_id),
                )),
                _ => None,
            };

            if let Some((event, facility_id)) = credit_event {
                message.inject_trace_parent();
                Span::current().record("handled", true);
                Span::current().record("event_type", collection_event.as_ref());

                let mut repayment_plan = self.repo.load(facility_id).await?;
                repayment_plan.process_event(sequence, &event, clock.now());
                self.repo
                    .persist_in_tx(db, facility_id, repayment_plan)
                    .await?;
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl<E> JobRunner for RepaymentPlanProjectionJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent> + OutboxEventMarker<CoreCreditCollectionEvent>,
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
                            self.process_message(&mut db, &message, state.sequence, current_job.clock())
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
