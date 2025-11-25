//! Partial Liquidation job monitors a running partial liquidation
//! process. In particular, it is overwatching the actual liquidation
//! of Bitcoins and is waiting for balance updates on relevant
//! accounts.

use serde::{Deserialize, Serialize};

use cala_ledger::{
    AccountId,
    outbox::{OutboxEvent, OutboxEventPayload},
};
use job::Jobs;
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use crate::{
    CollateralAction, CollateralizationState, CoreCreditEvent, LiquidationProcessId,
    liquidation_process::LiquidationProcessRepo,
};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PartialLiquidationJobConfig<E> {
    pub receivable_account_id: AccountId,
    pub liquidation_process_id: LiquidationProcessId,
    pub _phantom: std::marker::PhantomData<E>,
}

pub struct PartialLiquidationJobRunner<E: OutboxEventMarker<CoreCreditEvent>> {
    config: PartialLiquidationJobConfig<E>,
    outbox: Outbox<E>,
    jobs: Jobs,
    liquidation_process_repo: LiquidationProcessRepo<E>,
}

impl<E: OutboxEventMarker<CoreCreditEvent>> PartialLiquidationJobRunner<E> {
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
