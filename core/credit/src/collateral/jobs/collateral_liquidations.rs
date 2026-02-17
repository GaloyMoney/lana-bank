use tracing::{Span, instrument};

use std::sync::Arc;

use core_custody::CoreCustodyEvent;
use es_entity::{DbOp, Idempotent};
use governance::GovernanceEvent;
use money::{Satoshis, UsdCents};
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;

use super::{
    super::repo::CollateralRepo,
    liquidation_payment::{LiquidationPaymentJobConfig, LiquidationPaymentJobSpawner},
};
use crate::{
    CoreCreditEvent,
    collateral::ledger::LiquidationProceedsAccountIds,
    primitives::{CalaAccountId, CollateralId, PriceOfOneBTC},
};

pub const CREDIT_FACILITY_LIQUIDATIONS_JOB: JobType =
    JobType::new("outbox.credit-facility-liquidations");

pub struct CreditFacilityLiquidationsHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    repo: Arc<CollateralRepo<E>>,
    liquidation_proceeds_omnibus_account_id: CalaAccountId,
    liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
}

impl<E> CreditFacilityLiquidationsHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        repo: Arc<CollateralRepo<E>>,
        liquidation_proceeds_omnibus_account_id: CalaAccountId,
        liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
    ) -> Self {
        Self {
            repo,
            liquidation_proceeds_omnibus_account_id,
            liquidation_payment_job_spawner,
        }
    }
}

impl<E> OutboxEventHandler<E> for CreditFacilityLiquidationsHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    #[instrument(name = "outbox.core_credit.collateral_liquidations.process_message_in_op", parent = None, skip(self, op, event), fields(seq = %event.sequence, handled = false, event_type = tracing::field::Empty))]
    async fn handle_persistent(
        &self,
        op: &mut DbOp<'_>,
        event: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(e @ CoreCreditEvent::PartialLiquidationInitiated { entity }) = event.as_event()
        {
            Span::current().record("handled", true);
            Span::current().record("event_type", e.as_ref());

            let trigger = entity
                .liquidation_trigger
                .as_ref()
                .expect("liquidation_trigger must be set for PartialLiquidationInitiated");
            self.create_if_not_exist_in_op(
                op,
                entity.collateral_id,
                trigger.trigger_price,
                trigger.initially_expected_to_receive,
                trigger.initially_estimated_to_liquidate,
            )
            .await?;
        }
        Ok(())
    }
}

impl<E> CreditFacilityLiquidationsHandler<E>
where
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    #[instrument(
        name = "credit.liquidation.create_if_not_exist_in_op",
        skip(self, db),
        err
    )]
    pub async fn create_if_not_exist_in_op(
        &self,
        db: &mut DbOp<'_>,
        collateral_id: CollateralId,
        trigger_price: PriceOfOneBTC,
        initially_expected_to_receive: UsdCents,
        initially_estimated_to_liquidate: Satoshis,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut collateral = self.repo.find_by_id_in_op(&mut *db, collateral_id).await?;

        let liquidation_proceeds_account_ids = LiquidationProceedsAccountIds::new(
            &collateral.account_ids,
            &collateral.facility_ledger_account_ids_for_liquidation,
            self.liquidation_proceeds_omnibus_account_id,
        );

        let liquidation_id = if let Idempotent::Executed(id) = collateral
            .record_liquidation_started(
                trigger_price,
                initially_expected_to_receive,
                initially_estimated_to_liquidate,
                liquidation_proceeds_account_ids,
            ) {
            id
        } else {
            return Ok(());
        };

        self.repo.update_in_op(db, &mut collateral).await?;

        self.liquidation_payment_job_spawner
            .spawn_in_op(
                db,
                job::JobId::new(),
                LiquidationPaymentJobConfig::<E> {
                    liquidation_id,
                    collateral_id,
                    credit_facility_id: collateral.credit_facility_id,
                    _phantom: std::marker::PhantomData,
                },
            )
            .await?;

        Ok(())
    }
}
