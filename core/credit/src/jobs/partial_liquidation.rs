//! Partial Liquidation job monitors a running partial liquidation
//! process. In particular, it is overwatching the actual liquidation
//! of Bitcoins and is waiting for balance updates on relevant
//! accounts.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use cala_ledger::{
    AccountId,
    outbox::{OutboxEvent, OutboxEventPayload},
};
use job::*;
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{
    CollateralAction, CollateralizationState, CoreCreditEvent, LiquidationProcessId,
    liquidation_process::LiquidationProcessRepo,
};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PartialLiquidationJobConfig<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    pub receivable_account_id: AccountId,
    pub liquidation_process_id: LiquidationProcessId,
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
    jobs: Jobs,
    liquidation_process_repo: LiquidationProcessRepo<E>,
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
            jobs: self.jobs.clone(),
            liquidation_process_repo: self.liquidation_process_repo.clone(),
        }))
    }
}

pub struct PartialLiquidationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    config: PartialLiquidationJobConfig<E>,
    outbox: Outbox<E>,
    jobs: Jobs,
    liquidation_process_repo: LiquidationProcessRepo<E>,
}

#[async_trait]
impl<E> JobRunner for PartialLiquidationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        todo!()
    }
}

impl<E> PartialLiquidationJobRunner<E>
where
    E: OutboxEventMarker<CoreCreditEvent>,
{
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use CoreCreditEvent::*;

        match &message.as_event() {
            Some(FacilityCollateralUpdated {
                action: CollateralAction::Remove,
                ledger_tx_id,
                abs_diff,
                ..
            }) => {
                // change liquidation process status
                let mut x = self
                    .liquidation_process_repo
                    .find_by_id(self.config.liquidation_process_id)
                    .await?;

                x.record_collateral_sent(*abs_diff, *ledger_tx_id);

                self.liquidation_process_repo.update(&mut x).await?;

                todo!()
            }
            Some(PartialLiquidationSatisfied {
                credit_facility_id,
                amount,
            }) => {
                // record payment
                todo!()
            }
            Some(FacilityRepaymentRecorded {
                credit_facility_id,
                obligation_id,
                obligation_type,
                payment_id,
                amount,
                recorded_at,
                effective,
            }) => {
                // complete liquidation
                todo!()
            }
            _ => {}
        }

        Ok(())
    }

    async fn process_ledger_message(
        &self,
        message: &OutboxEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &message.payload {
            OutboxEventPayload::BalanceUpdated { balance, .. }
                if balance.account_id == self.config.receivable_account_id =>
            {
                let mut x = self
                    .liquidation_process_repo
                    .find_by_id(self.config.liquidation_process_id)
                    .await?;

                x.record_repayment_received(todo!(), todo!());

                self.liquidation_process_repo.update(&mut x).await?;

                todo!()
            }
            _ => {}
        }

        Ok(())
    }
}
