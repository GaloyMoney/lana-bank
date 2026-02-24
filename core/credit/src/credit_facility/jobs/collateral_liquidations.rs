use std::sync::Arc;

use tracing::{Span, instrument};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_custody::CoreCustodyEvent;
use es_entity::DbOp;
use governance::GovernanceEvent;
use obix::out::{OutboxEventHandler, OutboxEventMarker, PersistentOutboxEvent};

use job::JobType;

use super::liquidation_payment::{LiquidationPaymentJobConfig, LiquidationPaymentJobSpawner};
use crate::{
    CoreCreditCollectionEvent, CoreCreditEvent,
    collateral::{Collaterals, SecuredLoanId},
    primitives::{CoreCreditAction, CoreCreditObject},
};

pub const CREDIT_FACILITY_LIQUIDATIONS_JOB: JobType =
    JobType::new("outbox.credit-facility-liquidations");

pub struct CreditFacilityLiquidationsHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    collaterals: Arc<Collaterals<Perms, E>>,
    liquidation_proceeds_omnibus_account_id: crate::CalaAccountId,
    liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
}

impl<Perms, E> CreditFacilityLiquidationsHandler<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        collaterals: Arc<Collaterals<Perms, E>>,
        liquidation_proceeds_omnibus_account_id: crate::CalaAccountId,
        liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
    ) -> Self {
        Self {
            collaterals,
            liquidation_proceeds_omnibus_account_id,
            liquidation_payment_job_spawner,
        }
    }
}

impl<Perms, E> OutboxEventHandler<E> for CreditFacilityLiquidationsHandler<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditAction> + From<core_credit_collection::CoreCreditCollectionAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditObject> + From<core_credit_collection::CoreCreditCollectionObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
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

            let result: Option<SecuredLoanId> = self
                .collaterals
                .record_liquidation_started_in_op(
                    op,
                    entity.collateral_id,
                    trigger.liquidation_id,
                    trigger.trigger_price,
                    trigger.initially_expected_to_receive,
                    trigger.initially_estimated_to_liquidate,
                    self.liquidation_proceeds_omnibus_account_id,
                )
                .await?;

            if let Some(secured_loan_id) = result {
                self.liquidation_payment_job_spawner
                    .spawn_in_op(
                        op,
                        job::JobId::new(),
                        LiquidationPaymentJobConfig::<E> {
                            liquidation_id: trigger.liquidation_id,
                            collateral_id: entity.collateral_id,
                            credit_facility_id: secured_loan_id.into(),
                            _phantom: std::marker::PhantomData,
                        },
                    )
                    .await?;
            }
        }
        Ok(())
    }
}
