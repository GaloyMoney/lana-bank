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

use super::super::repo::CollateralRepo;
use crate::{CoreCreditEvent, CreditFacilityId, LiquidationId, liquidation::LiquidationRepo};

#[derive(Default, Clone, Deserialize, Serialize)]
struct PartialLiquidationJobData {
    sequence: EventSequence,
}

#[derive(Serialize, Deserialize)]
pub struct PartialLiquidationJobConfig<E> {
    pub liquidation_id: LiquidationId,
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> Clone for PartialLiquidationJobConfig<E> {
    fn clone(&self) -> Self {
        Self {
            liquidation_id: self.liquidation_id,
            credit_facility_id: self.credit_facility_id,
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
    liquidation_repo: Arc<LiquidationRepo<E>>,
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
        liquidation_repo: Arc<LiquidationRepo<E>>,
        collateral_repo: Arc<CollateralRepo<E>>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            liquidation_repo,
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
            liquidation_repo: self.liquidation_repo.clone(),
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
    liquidation_repo: Arc<LiquidationRepo<E>>,
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
                                .liquidation_repo
                                .begin_op()
                                .await?;

                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut db, &state)
                                .await?;

                            let next = self.process_message(&mut db, message.as_ref()).await?;

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
    #[instrument(
        name = "outbox.core_credit.partial_liquidation.complete_liquidation",
        skip(self, db)
    )]
    async fn complete_liquidation(
        &self,
        db: &mut DbOp<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut liquidation = self
            .liquidation_repo
            .find_by_id(self.config.liquidation_id)
            .await?;

        if liquidation.complete().did_execute() {
            self.liquidation_repo
                .update_in_op(db, &mut liquidation)
                .await?;

            let mut collateral = self
                .collateral_repo
                .find_by_id_in_op(&mut *db, liquidation.collateral_id)
                .await?;
            let _ = collateral.record_liquidation_completed(liquidation.id);
            self.collateral_repo
                .update_in_op(db, &mut collateral)
                .await?;
        }

        Ok(())
    }

    #[instrument(name = "outbox.core_credit.partial_liquidation.process_message", parent = None, skip(self, message, db), fields(payment_id, seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message(
        &self,
        db: &mut DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<ControlFlow<()>, Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        match &message.as_event() {
            Some(
                event @ PartialLiquidationProceedsReceived {
                    liquidation_id,
                    payment_id,
                    ..
                },
            ) if *liquidation_id == self.config.liquidation_id => {
                Span::current().record("handled", true);
                Span::current().record("event_type", event.as_ref());
                Span::current().record("payment_id", tracing::field::display(payment_id));

                self.complete_liquidation(db).await?;

                Ok(ControlFlow::Break(()))
            }
            _ => Ok(ControlFlow::Continue(())),
        }
    }
}

pub type PartialLiquidationJobSpawner<E> = JobSpawner<PartialLiquidationJobConfig<E>>;
