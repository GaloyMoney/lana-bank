use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_credit_collection::CoreCreditCollectionEvent;
use core_custody::CoreCustodyEvent;
use governance::GovernanceEvent;
use job::*;
use money::{Satoshis, UsdCents};
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use super::liquidation_payment::{LiquidationPaymentJobConfig, LiquidationPaymentJobSpawner};
use crate::{
    CalaAccountId, CoreCreditEvent,
    collateral::{Collaterals, public::CoreCreditCollateralEvent},
    primitives::{CollateralId, LiquidationId, PriceOfOneBTC},
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecordLiquidationConfig {
    pub collateral_id: CollateralId,
    pub liquidation_id: LiquidationId,
    pub trigger_price: PriceOfOneBTC,
    pub initially_expected_to_receive: UsdCents,
    pub initially_estimated_to_liquidate: Satoshis,
    #[serde(default)]
    pub trace_context: Option<tracing_utils::persistence::SerializableTraceContext>,
}

pub const RECORD_LIQUIDATION_COMMAND: JobType = JobType::new("command.credit.record-liquidation");

pub struct RecordLiquidationJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    collaterals: Arc<Collaterals<Perms, E>>,
    liquidation_proceeds_omnibus_account_id: CalaAccountId,
    liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
}

impl<Perms, E> RecordLiquidationJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        collaterals: Arc<Collaterals<Perms, E>>,
        liquidation_proceeds_omnibus_account_id: CalaAccountId,
        liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
    ) -> Self {
        Self {
            collaterals,
            liquidation_proceeds_omnibus_account_id,
            liquidation_payment_job_spawner,
        }
    }
}

impl<Perms, E> JobInitializer for RecordLiquidationJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<crate::primitives::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<crate::collateral::primitives::CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<crate::primitives::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<crate::collateral::primitives::CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Config = RecordLiquidationConfig;

    fn job_type(&self) -> JobType {
        RECORD_LIQUIDATION_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(RecordLiquidationJobRunner {
            config: job.config()?,
            collaterals: self.collaterals.clone(),
            liquidation_proceeds_omnibus_account_id: self.liquidation_proceeds_omnibus_account_id,
            liquidation_payment_job_spawner: self.liquidation_payment_job_spawner.clone(),
        }))
    }
}

struct RecordLiquidationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    config: RecordLiquidationConfig,
    collaterals: Arc<Collaterals<Perms, E>>,
    liquidation_proceeds_omnibus_account_id: CalaAccountId,
    liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
}

#[async_trait]
impl<Perms, E> JobRunner for RecordLiquidationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<crate::primitives::CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<crate::collateral::primitives::CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<crate::primitives::CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<crate::collateral::primitives::CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "credit.record_liquidation_job.process_command",
        skip(self, current_job)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        if let Some(ref ctx) = self.config.trace_context {
            tracing_utils::persistence::set_parent(ctx);
        }
        let mut op = current_job.begin_op().await?;

        let result = self
            .collaterals
            .record_liquidation_started_in_op(
                &mut op,
                self.config.collateral_id,
                self.config.liquidation_id,
                self.config.trigger_price,
                self.config.initially_expected_to_receive,
                self.config.initially_estimated_to_liquidate,
                self.liquidation_proceeds_omnibus_account_id,
            )
            .await?;

        if let Some(secured_loan_id) = result {
            self.liquidation_payment_job_spawner
                .spawn_in_op(
                    &mut op,
                    JobId::new(),
                    LiquidationPaymentJobConfig::<E> {
                        liquidation_id: self.config.liquidation_id,
                        collateral_id: self.config.collateral_id,
                        credit_facility_id: secured_loan_id.into(),
                        _phantom: std::marker::PhantomData,
                    },
                )
                .await?;
        }

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
