use std::{ops::ControlFlow, sync::Arc};

use async_trait::async_trait;
use futures::StreamExt as _;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use core_custody::CoreCustodyEvent;
use es_entity::DbOp;
use governance::GovernanceEvent;
use job::*;
use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{
    CollateralId, CoreCreditEvent, collateral::CollateralRepo, credit_facility::CreditFacilityRepo,
};

#[derive(Default, Clone, Deserialize, Serialize)]
struct PartialLiquidationJobData {
    sequence: EventSequence,
}

#[derive(Serialize, Deserialize)]
pub struct PartialLiquidationJobConfig<E> {
    pub collateral_id: CollateralId,
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for PartialLiquidationJobConfig<E> {
    fn clone(&self) -> Self {
        Self {
            collateral_id: self.collateral_id,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub struct PartialLiquidationInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    collateral_repo: Arc<CollateralRepo<E>>,
}

impl<E> PartialLiquidationInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        credit_facility_repo: Arc<CreditFacilityRepo<E>>,
        collateral_repo: Arc<CollateralRepo<E>>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            credit_facility_repo,
            collateral_repo,
        }
    }
}

const PARTIAL_LIQUIDATION_JOB: JobType = JobType::new("outbox.partial-liquidation");

impl<E> JobInitializer for PartialLiquidationInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Config = PartialLiquidationJobConfig<E>;

    fn job_type(&self) -> JobType {
        PARTIAL_LIQUIDATION_JOB
    }

    fn init(
        &self,
        job: &job::Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn job::JobRunner>, Box<dyn std::error::Error>> {
        let config: PartialLiquidationJobConfig<E> = job.config()?;
        Ok(Box::new(PartialLiquidationJobRunner::<E> {
            config,
            outbox: self.outbox.clone(),
            credit_facility_repo: self.credit_facility_repo.clone(),
            collateral_repo: self.collateral_repo.clone(),
        }))
    }
}

pub struct PartialLiquidationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    config: PartialLiquidationJobConfig<E>,
    outbox: Outbox<E>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    collateral_repo: Arc<CollateralRepo<E>>,
}

#[async_trait]
impl<E> JobRunner for PartialLiquidationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<PartialLiquidationJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %PARTIAL_LIQUIDATION_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            let mut db = self
                                .collateral_repo
                                .begin_op_with_clock(current_job.clock())
                                .await?;

                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut db, &state)
                                .await?;

                            let next = self.process_message(&mut db, message.as_ref(), current_job.clock()).await?;

                            db.commit().await?;

                            if next.is_break() {
                                // If the partial liquidation has been completed,
                                // terminate the job, too.
                                return Ok(JobCompletion::Complete);
                            }
                        }
                        None => return Ok(JobCompletion::RescheduleNow)
                    }
                }
            }
        }
    }
}

impl<E> PartialLiquidationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    #[instrument(name = "outbox.core_credit.partial_liquidation.process_message", parent = None, skip(self, message, db), fields(payment_id, seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        db: &mut DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
        clock: &es_entity::clock::ClockHandle,
    ) -> Result<ControlFlow<()>, Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        match &message.as_event() {
            Some(
                event @ PartialLiquidationProceedsReceived {
                    collateral_id,
                    payment_id,
                    ..
                },
            ) if *collateral_id == self.config.collateral_id => {
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());
                Span::current().record("payment_id", tracing::field::display(payment_id));

                let mut collateral = self
                    .collateral_repo
                    .find_by_id_in_op(&mut *db, collateral_id)
                    .await?;

                if collateral.exit_liquidation().did_execute() {
                    self.collateral_repo
                        .update_in_op(db, &mut collateral)
                        .await?;
                    Ok(ControlFlow::Break(()))
                } else {
                    Ok(ControlFlow::Continue(()))
                }
            }
            _ => Ok(ControlFlow::Continue(())),
        }
    }
}

pub type PartialLiquidationJobSpawner<E> = JobSpawner<PartialLiquidationJobConfig<E>>;
