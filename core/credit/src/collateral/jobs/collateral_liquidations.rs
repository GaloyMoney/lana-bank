use async_trait::async_trait;
use futures::StreamExt as _;
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{Span, instrument};

use std::sync::Arc;

use core_custody::CoreCustodyEvent;
use es_entity::{DbOp, Idempotent};
use governance::GovernanceEvent;
use job::*;
use money::{Satoshis, UsdCents};
use obix::EventSequence;
use obix::out::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use super::{
    super::repo::CollateralRepo,
    liquidation_payment::{LiquidationPaymentJobConfig, LiquidationPaymentJobSpawner},
};
use crate::{
    CoreCreditEvent,
    collateral::error::CollateralError,
    primitives::{CollateralId, CreditFacilityId, PriceOfOneBTC},
};

#[derive(Default, Clone, Deserialize, Serialize)]
struct CreditFacilityLiquidationsJobData {
    sequence: EventSequence,
}

#[derive(Serialize, Deserialize)]
pub struct CreditFacilityLiquidationsJobConfig;

pub struct CreditFacilityLiquidationsInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    repo: Arc<CollateralRepo<E>>,
    liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
}

impl<E> CreditFacilityLiquidationsInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        repo: Arc<CollateralRepo<E>>,
        liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            repo,
            liquidation_payment_job_spawner,
        }
    }
}

const CREDIT_FACILITY_LIQUIDATIONS_JOB: JobType =
    JobType::new("outbox.credit-facility-liquidations");
impl<E> JobInitializer for CreditFacilityLiquidationsInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Config = CreditFacilityLiquidationsJobConfig;
    fn job_type(&self) -> JobType {
        CREDIT_FACILITY_LIQUIDATIONS_JOB
    }

    fn init(
        &self,
        _job: &job::Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn job::JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityLiquidationsJobRunner::<E> {
            outbox: self.outbox.clone(),
            repo: self.repo.clone(),
            liquidation_payment_job_spawner: self.liquidation_payment_job_spawner.clone(),
        }))
    }
}

pub struct CreditFacilityLiquidationsJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    repo: Arc<CollateralRepo<E>>,
    liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
}

#[async_trait]
impl<E> JobRunner for CreditFacilityLiquidationsJobRunner<E>
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
            .execution_state::<CreditFacilityLiquidationsJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %CREDIT_FACILITY_LIQUIDATIONS_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {

                            let mut db = self
                                .repo
                                .begin_op()
                                .await?;
                            self.process_message_in_op(&mut db, message.as_ref()).await?;
                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut db, &state)
                                .await?;
                            db.commit().await?;
                        }
                        None => return Ok(JobCompletion::RescheduleNow)
                    }
                }
            }
        }
    }
}

impl<E> CreditFacilityLiquidationsJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    #[instrument(name = "outbox.core_credit.collateral_liquidations.process_message_in_op", parent = None, skip(self, message, db), fields(seq = %message.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn process_message_in_op(
        &self,
        db: &mut DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(event @ CoreCreditEvent::PartialLiquidationInitiated { entity }) =
            message.as_event()
        {
            Span::current().record("handled", true);
            Span::current().record("event_type", event.as_ref());

            let trigger = entity
                .liquidation_trigger
                .as_ref()
                .expect("liquidation_trigger must be set for PartialLiquidationInitiated");
            self.create_if_not_exist_in_op(
                db,
                entity.collateral_id,
                entity.id,
                trigger.trigger_price,
                trigger.initially_expected_to_receive,
                trigger.initially_estimated_to_liquidate,
            )
            .await?;
        }
        Ok(())
    }

    #[instrument(
        name = "credit.liquidation.create_if_not_exist_in_op",
        skip(self, db),
        err
    )]
    pub async fn create_if_not_exist_in_op(
        &self,
        db: &mut DbOp<'_>,
        collateral_id: CollateralId,
        credit_facility_id: CreditFacilityId,
        trigger_price: PriceOfOneBTC,
        initially_expected_to_receive: UsdCents,
        initially_estimated_to_liquidate: Satoshis,
    ) -> Result<(), CollateralError> {
        let mut collateral = self.repo.find_by_id_in_op(&mut *db, collateral_id).await?;

        let liquidation_id = if let Idempotent::Executed(id) = collateral
            .record_liquidation_started(
                trigger_price,
                initially_expected_to_receive,
                initially_estimated_to_liquidate,
            ) {
            id
        } else {
            return Ok(());
        };

        self.repo.update_in_op(db, &mut collateral).await?;

        self.liquidation_payment_job_spawner
            .spawn_in_op(
                db,
                JobId::new(),
                LiquidationPaymentJobConfig::<E> {
                    liquidation_id,
                    credit_facility_id,
                    _phantom: std::marker::PhantomData,
                },
            )
            .await?;

        Ok(())
    }
}
