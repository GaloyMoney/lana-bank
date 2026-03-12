use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_credit_collateral::{self, CalaAccountId, CollateralId, Collaterals, LiquidationId};
use core_custody::CoreCustodyEvent;
use governance::GovernanceEvent;
use job::*;
use money::{Satoshis, UsdCents};

use crate::PriceOfOneBTC;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use super::liquidation_payment::{LiquidationPaymentJobConfig, LiquidationPaymentJobSpawner};
use crate::{
    CoreCreditCollectionEvent, CoreCreditEvent,
    primitives::{CoreCreditAction, CoreCreditObject},
};
use core_credit_collateral::{SecuredLoanId, public::CoreCreditCollateralEvent};

pub const RECORD_LIQUIDATION_STARTED_COMMAND: JobType =
    JobType::new("command.credit.record-liquidation-started");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecordLiquidationStartedConfig {
    pub collateral_id: CollateralId,
    pub liquidation_id: LiquidationId,
    pub trigger_price: PriceOfOneBTC,
    pub initially_expected_to_receive: UsdCents,
    pub initially_estimated_to_liquidate: Satoshis,
    pub liquidation_proceeds_omnibus_account_id: CalaAccountId,
}

pub struct RecordLiquidationStartedJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    collaterals: Arc<Collaterals<Perms, E>>,
    liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
}

impl<Perms, E> RecordLiquidationStartedJobInitializer<Perms, E>
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
        liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
    ) -> Self {
        Self {
            collaterals,
            liquidation_payment_job_spawner,
        }
    }
}

impl<Perms, E> JobInitializer for RecordLiquidationStartedJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_credit_collateral::primitives::CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_credit_collateral::primitives::CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Config = RecordLiquidationStartedConfig;

    fn job_type(&self) -> JobType {
        RECORD_LIQUIDATION_STARTED_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(RecordLiquidationStartedJobRunner {
            config: job.config()?,
            collaterals: self.collaterals.clone(),
            liquidation_payment_job_spawner: self.liquidation_payment_job_spawner.clone(),
        }))
    }
}

struct RecordLiquidationStartedJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    config: RecordLiquidationStartedConfig,
    collaterals: Arc<Collaterals<Perms, E>>,
    liquidation_payment_job_spawner: LiquidationPaymentJobSpawner<E>,
}

#[async_trait]
impl<Perms, E> JobRunner for RecordLiquidationStartedJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<core_credit_collateral::primitives::CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<core_credit_collateral::primitives::CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "credit.record_liquidation_started.process_command", skip_all)]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        let result: Option<SecuredLoanId> = self
            .collaterals
            .record_liquidation_started_in_op(
                &mut op,
                self.config.collateral_id,
                self.config.liquidation_id,
                self.config.trigger_price,
                self.config.initially_expected_to_receive,
                self.config.initially_estimated_to_liquidate,
                self.config.liquidation_proceeds_omnibus_account_id,
            )
            .await?;

        if let Some(secured_loan_id) = result {
            self.liquidation_payment_job_spawner
                .spawn_in_op(
                    &mut op,
                    job::JobId::new(),
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
