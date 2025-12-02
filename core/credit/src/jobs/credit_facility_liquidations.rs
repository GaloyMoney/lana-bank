use async_trait::async_trait;
use core_custody::CoreCustodyEvent;
use es_entity::DbOp;
use futures::StreamExt as _;
use governance::GovernanceEvent;
use job::*;
use outbox::{EventSequence, Outbox, OutboxEventMarker, PersistentOutboxEvent};
use serde::{Deserialize, Serialize};

use crate::CoreCreditEvent;
use crate::jobs::partial_liquidation;
use crate::liquidation::{Liquidations, NewLiquidation};

#[derive(Default, Clone, Deserialize, Serialize)]
struct CreditFacilityLiquidationsJobData {
    sequence: EventSequence,
}

pub struct CreditFacilityLiquidationsInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    jobs: Jobs,
    liquidations: Liquidations<E>,
}

impl<E> CreditFacilityLiquidationsInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(outbox: &Outbox<E>, jobs: &Jobs, liquidations: &Liquidations<E>) -> Self {
        Self {
            outbox: outbox.clone(),
            jobs: jobs.clone(),
            liquidations: liquidations.clone(),
        }
    }
}

const CREDIT_FACILITY_LIQUDATIONS_JOB: JobType =
    JobType::new("outbox.credit-facility-liquidations");
impl<E> JobInitializer for CreditFacilityLiquidationsInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_LIQUDATIONS_JOB
    }

    fn init(&self, _job: &job::Job) -> Result<Box<dyn job::JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityLiquidationsJobRunner::<E> {
            outbox: self.outbox.clone(),
            jobs: self.jobs.clone(),
            liquidations: self.liquidations.clone(),
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
    jobs: Jobs,
    liquidations: Liquidations<E>,
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

        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            let mut db = self.liquidations.begin_op().await?;
            self.process_message(&mut db, message.as_ref()).await?;
            state.sequence = message.sequence;
            current_job
                .update_execution_state_in_op(&mut db, &state)
                .await?;

            db.commit().await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

impl<E> CreditFacilityLiquidationsJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    async fn process_message(
        &self,
        db: &mut DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(CoreCreditEvent::PartialLiquidationInitiated {
            liquidation_id,
            credit_facility_id,
            receivable_account_id,
            trigger_price,
            initially_expected_to_receive,
            initially_estimated_to_liquidate,
        }) = message.as_event()
        {
            let maybe_new_liqudation = self
                .liquidations
                .create_if_not_exist_for_facility_in_op(
                    db,
                    *credit_facility_id,
                    NewLiquidation::builder()
                        .id(*liquidation_id)
                        .credit_facility_id(*credit_facility_id)
                        .receivable_account_id(*receivable_account_id)
                        .trigger_price(*trigger_price)
                        .initially_expected_to_receive(*initially_expected_to_receive)
                        .initially_estimated_to_liquidate(*initially_estimated_to_liquidate)
                        .build()
                        .expect("Could not build new liquidation"),
                )
                .await?;

            if let Some(liquidation) = maybe_new_liqudation {
                self.jobs
                    .create_and_spawn_in_op(
                        db,
                        JobId::new(),
                        partial_liquidation::PartialLiquidationJobConfig::<E> {
                            liquidation_id: liquidation.id,
                            credit_facility_id: *credit_facility_id,
                            _phantom: std::marker::PhantomData,
                        },
                    )
                    .await?;
            }
        }
        Ok(())
    }
}
