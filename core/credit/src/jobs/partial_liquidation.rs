use std::ops::ControlFlow;

use async_trait::async_trait;
use futures::StreamExt as _;
use serde::{Deserialize, Serialize};

use job::*;
use outbox::*;

use crate::{CoreCreditEvent, CreditFacilityId, LiquidationId, liquidation::Liquidations};

#[derive(Default, Clone, Deserialize, Serialize)]
struct PartialLiquidationJobData {
    sequence: EventSequence,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PartialLiquidationJobConfig<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub liquidation_id: LiquidationId,
    pub credit_facility_id: CreditFacilityId,
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> JobConfig for PartialLiquidationJobConfig<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Initializer = PartialLiquidationInit<E>;
}

pub struct PartialLiquidationInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    outbox: Outbox<E>,
    liquidations: Liquidations<E>,
}

impl<E> PartialLiquidationInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(outbox: &Outbox<E>, liquidations: &Liquidations<E>) -> Self {
        Self {
            outbox: outbox.clone(),
            liquidations: liquidations.clone(),
        }
    }
}

const PARTIAL_LIQUIDATION_JOB: JobType = JobType::new("outbox.partial-liquidation");
impl<E> JobInitializer for PartialLiquidationInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        PARTIAL_LIQUIDATION_JOB
    }

    fn init(&self, job: &job::Job) -> Result<Box<dyn job::JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(PartialLiquidationJobRunner::<E> {
            config: job.config()?,
            outbox: self.outbox.clone(),
            liquidations: self.liquidations.clone(),
        }))
    }
}

pub struct PartialLiquidationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: PartialLiquidationJobConfig<E>,
    outbox: Outbox<E>,
    liquidations: Liquidations<E>,
}

#[async_trait]
impl<E> JobRunner for PartialLiquidationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<PartialLiquidationJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            let mut db = self.liquidations.begin_op().await?;

            let next = self.process_message(&mut db, message.as_ref()).await?;

            state.sequence = message.sequence;
            current_job
                .update_execution_state_in_op(&mut db, &state)
                .await?;

            db.commit().await?;

            if next.is_break() {
                return Ok(JobCompletion::Complete);
            }
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

impl<E> PartialLiquidationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    async fn process_message(
        &self,
        db: &mut es_entity::DbOp<'_>,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<ControlFlow<()>, Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        match &message.as_event() {
            Some(PartialLiquidationRepaymentAmountReceived {
                credit_facility_id, ..
            }) if *credit_facility_id == self.config.credit_facility_id => {
                let payment_id = crate::PaymentId::new();
                // TODO: let payment_id = credit::record_payment

                self.liquidations
                    .complete_in_op(db, self.config.liquidation_id, payment_id)
                    .await?;

                Ok(ControlFlow::Break(()))
            }
            _ => Ok(ControlFlow::Continue(())),
        }
    }
}
