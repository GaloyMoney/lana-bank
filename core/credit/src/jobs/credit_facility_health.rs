//! Credit Facility Health listens to changes in collateralization
//! state of credit facilities and initiates a partial liquidation of
//! credit facility whose CVL drops below liquidation threshold
//! (i. e. became unhealthy), unless this credit facility is already
//! in an active liquidation.
//!
//! All other state changes are ignored by this job.

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
use crate::liquidation_process::Liquidations;

#[derive(Default, Clone, Deserialize, Serialize)]
struct CreditFacilityHealthJobData {
    sequence: EventSequence,
}

pub struct CreditFacilityHealthInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    outbox: Outbox<E>,
    jobs: Jobs,
    liquidations: Liquidations<E>,
}

impl<E> CreditFacilityHealthInit<E>
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

const CREDIT_FACILITY_HEALTH_JOB: JobType = JobType::new("outbox.credit-facility-health");
impl<E> JobInitializer for CreditFacilityHealthInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_HEALTH_JOB
    }

    fn init(&self, _job: &job::Job) -> Result<Box<dyn job::JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityHealthJobRunner::<E> {
            outbox: self.outbox.clone(),
            jobs: self.jobs.clone(),
            liquidations: self.liquidations.clone(),
        }))
    }
}

pub struct CreditFacilityHealthJobRunner<E>
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
impl<E> JobRunner for CreditFacilityHealthJobRunner<E>
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
            .execution_state::<CreditFacilityHealthJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            let mut db = self.liquidations.begin_op().await?;
            self.process_message(message.as_ref(), &mut db).await?;
            state.sequence = message.sequence;
            current_job
                .update_execution_state_in_op(&mut db, &state)
                .await?;

            db.commit().await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

impl<E> CreditFacilityHealthJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
        db: &mut DbOp<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(event) = message.as_event() {
            if let CoreCreditEvent::PartialLiquidationInitiated {
                liquidation_process_id,
                credit_facility_id,
                receivable_account_id,
                trigger_price,
                initially_expected_to_receive,
                initially_estimated_to_liquidate,
            } = event
            {
                if let Some(liquidation) = self
                    .liquidations
                    .create_if_not_exist_in_op(
                        db,
                        *liquidation_process_id,
                        *credit_facility_id,
                        *receivable_account_id,
                        *trigger_price,
                        *initially_expected_to_receive,
                        *initially_estimated_to_liquidate,
                    )
                    .await?
                {
                    self.jobs
                        .create_and_spawn_in_op(
                            db,
                            JobId::new(),
                            partial_liquidation::PartialLiquidationJobConfig::<E> {
                                liquidation_process_id: liquidation.id,
                                credit_facility_id: *credit_facility_id,
                                _phantom: std::marker::PhantomData,
                            },
                        )
                        .await?;
                }
            }
        }

        Ok(())
    }
}
