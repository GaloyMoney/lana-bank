//! Partial Liquidation job monitors a running partial liquidation
//! process. In particular, it is overwatching the actual liquidation
//! of Bitcoins and is waiting for balance updates on relevant
//! accounts.

use async_trait::async_trait;
use futures::StreamExt as _;
use outbox::OutboxEventMarker;
use serde::{Deserialize, Serialize};

use cala_ledger::{AccountId, CalaLedger, outbox::*};
use job::*;

use crate::{CoreCreditEvent, LiquidationProcessId, liquidation_process::Liquidations};

#[derive(Default, Clone, Deserialize, Serialize)]
struct PartialLiquidationCalaJobData {
    sequence: EventSequence,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PartialLiquidationCalaJobConfig<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub receivable_account_id: AccountId,
    pub liquidation_process_id: LiquidationProcessId,
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E> JobConfig for PartialLiquidationCalaJobConfig<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    type Initializer = PartialLiquidationCalaInit<E>;
}

pub struct PartialLiquidationCalaInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    cala: CalaLedger,
    liquidations: Liquidations<E>,
}

impl<E> PartialLiquidationCalaInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub fn new(cala: &CalaLedger, liquidations: &Liquidations<E>) -> Self {
        Self {
            cala: cala.clone(),
            liquidations: liquidations.clone(),
        }
    }
}

const PARTIAL_LIQUIDATION_CALA_JOB: JobType = JobType::new("outbox.partial-liquidation-cala");
impl<E> JobInitializer for PartialLiquidationCalaInit<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        PARTIAL_LIQUIDATION_CALA_JOB
    }

    fn init(&self, job: &job::Job) -> Result<Box<dyn job::JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(PartialLiquidationCalaJobRunner::<E> {
            config: job.config()?,
            cala: self.cala.clone(),
            liquidations: self.liquidations.clone(),
        }))
    }
}

pub struct PartialLiquidationCalaJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: PartialLiquidationCalaJobConfig<E>,
    cala: CalaLedger,
    liquidations: Liquidations<E>,
}

#[async_trait]
impl<E> JobRunner for PartialLiquidationCalaJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<PartialLiquidationCalaJobData>()?
            .unwrap_or_default();

        let mut stream = self
            .cala
            .register_outbox_listener(Some(state.sequence))
            .await?;

        while let Some(message) = stream.next().await {
            let mut db = self.liquidations.begin_op().await?;

            self.process_message(&mut db, &message).await?;

            state.sequence = message.sequence;
            current_job
                .update_execution_state_in_op(&mut db, &state)
                .await?;

            db.commit().await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

impl<E> PartialLiquidationCalaJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    async fn process_message(
        &self,
        db: &mut es_entity::DbOp<'_>,
        message: &OutboxEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &message.payload {
            // TransactionCreated with external ID or some other easily detectable value?
            OutboxEventPayload::BalanceUpdated { balance, .. }
                if balance.account_id == self.config.receivable_account_id =>
            {
                self.liquidations
                    .record_payment_from_liquidator_in_op(db, self.config.liquidation_process_id)
                    .await?;
            }
            _ => {}
        }

        Ok(())
    }
}
