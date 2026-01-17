use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use futures::StreamExt;
use std::sync::Arc;

use job::*;
use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{event::CoreCreditEvent, primitives::CreditFacilityId};

use super::super::repo::HistoryRepo;

#[derive(Default, Clone, Deserialize, Serialize)]
struct HistoryProjectionJobData {
    sequence: EventSequence,
}

pub struct HistoryProjectionJobRunner<E: OutboxEventMarker<CoreCreditEvent>> {
    outbox: Outbox<E>,
    repo: Arc<HistoryRepo>,
}

impl<E> HistoryProjectionJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    #[instrument(name = "outbox.core_credit.history_projection_job.process_message", parent = None, skip(self, message, db), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    #[allow(clippy::single_match)]
    async fn process_message(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        message: &PersistentOutboxEvent<E>,
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
                self.handle_event(db, message, event, *id).await?;
            }
            Some(event @ PendingCreditFacilityCollateralizationChanged { id, .. }) => {
                self.handle_event(db, message, event, *id).await?;
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
            )
            | Some(
                event @ PartialLiquidationProceedsReceived {
                    credit_facility_id: id,
                    ..
                },
            )
            | Some(
                event @ PartialLiquidationCollateralSentOut {
                    credit_facility_id: id,
                    ..
                },
            ) => {
                self.handle_event(db, message, event, *id).await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_event(
        &self,
        db: &mut sqlx::PgTransaction<'_>,
        message: &PersistentOutboxEvent<E>,
        event: &CoreCreditEvent,
        id: impl Into<CreditFacilityId>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let id = id.into();
        message.inject_trace_parent();
        Span::current().record("handled", true);
        Span::current().record("event_type", event.as_ref());
        let mut history = self.repo.load(id).await?;
        history.process_event(event);
        self.repo.persist_in_tx(db, id, history).await?;
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
        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %HISTORY_PROJECTION,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            let mut db = self.repo.begin().await?;
                            self.process_message(&mut db, message.as_ref()).await?;
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

pub struct HistoryProjectionInit<E: OutboxEventMarker<CoreCreditEvent>> {
    outbox: Outbox<E>,
    repo: Arc<HistoryRepo>,
}

impl<E> HistoryProjectionInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(outbox: &Outbox<E>, repo: Arc<HistoryRepo>) -> Self {
        Self {
            outbox: outbox.clone(),
            repo,
        }
    }
}

const HISTORY_PROJECTION: JobType = JobType::new("outbox.credit-facility-history-projection");

#[derive(Serialize, Deserialize)]
pub struct HistoryProjectionConfig<E> {
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for HistoryProjectionConfig<E> {
    fn clone(&self) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> JobInitializer for HistoryProjectionInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Config = HistoryProjectionConfig<E>;

    fn job_type(&self) -> JobType {
        HISTORY_PROJECTION
    }

    fn init(
        &self,
        _: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(HistoryProjectionJobRunner {
            outbox: self.outbox.clone(),
            repo: self.repo.clone(),
        }))
    }

    fn retry_on_error_settings(&self) -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}
