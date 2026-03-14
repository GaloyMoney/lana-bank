use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use core_credit_collateral::{
    CollateralId, Collaterals, CoreCreditCollateralAction, CoreCreditCollateralObject,
    LiquidationId, public::CoreCreditCollateralEvent,
};
use core_credit_collection::PaymentId;
use core_credit_collection::{
    BeneficiaryId, CoreCreditCollection, CoreCreditCollectionEvent, PaymentLedgerAccountIds,
};
use core_custody::CoreCustodyEvent;
use governance::GovernanceEvent;
use job::*;
use old_money::UsdCents;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    credit_facility::CreditFacilityRepo,
    primitives::{CoreCreditAction, CoreCreditObject, CreditFacilityId},
    public::CoreCreditEvent,
};

pub const RECORD_LIQUIDATION_PROCEEDS_COMMAND: JobType =
    JobType::new("command.credit.record-liquidation-proceeds");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecordLiquidationProceedsConfig {
    pub liquidation_id: LiquidationId,
    pub collateral_id: CollateralId,
    pub credit_facility_id: CreditFacilityId,
    pub amount: UsdCents,
    pub payment_id: PaymentId,
}

pub struct RecordLiquidationProceedsJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    collections: Arc<CoreCreditCollection<Perms, E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
}

impl<Perms, E> RecordLiquidationProceedsJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        collections: Arc<CoreCreditCollection<Perms, E>>,
        collaterals: Arc<Collaterals<Perms, E>>,
        credit_facility_repo: Arc<CreditFacilityRepo<E>>,
    ) -> Self {
        Self {
            collections,
            collaterals,
            credit_facility_repo,
        }
    }
}

impl<Perms, E> JobInitializer for RecordLiquidationProceedsJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Config = RecordLiquidationProceedsConfig;

    fn job_type(&self) -> JobType {
        RECORD_LIQUIDATION_PROCEEDS_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(RecordLiquidationProceedsJobRunner {
            config: job.config()?,
            collections: self.collections.clone(),
            collaterals: self.collaterals.clone(),
            credit_facility_repo: self.credit_facility_repo.clone(),
        }))
    }
}

struct RecordLiquidationProceedsJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    config: RecordLiquidationProceedsConfig,
    collections: Arc<CoreCreditCollection<Perms, E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    credit_facility_repo: Arc<CreditFacilityRepo<E>>,
}

#[async_trait]
impl<Perms, E> JobRunner for RecordLiquidationProceedsJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<core_credit_collection::CoreCreditCollectionAction>
        + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<core_credit_collection::CoreCreditCollectionObject>
        + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "credit.record_liquidation_proceeds.process_command", skip_all)]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        let facility_ids = self
            .collaterals
            .liquidation_ledger_account_ids_in_op(&mut op, self.config.collateral_id)
            .await?;

        let payment_ledger_account_ids = PaymentLedgerAccountIds {
            facility_payment_holding_account_id: facility_ids.payment_holding_account_id,
            facility_uncovered_outstanding_account_id: facility_ids
                .uncovered_outstanding_account_id,
            payment_source_account_id: facility_ids.proceeds_from_liquidation_account_id.into(),
        };

        let beneficiary_id: BeneficiaryId = self.config.credit_facility_id.into();

        self.collections
            .payments()
            .record_in_op(
                &mut op,
                self.config.payment_id,
                beneficiary_id,
                payment_ledger_account_ids,
                self.config.amount,
                current_job.clock().today(),
                &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(
                    crate::primitives::COLLATERALIZATION_SYNC,
                ),
            )
            .await?;

        let mut credit_facility = self
            .credit_facility_repo
            .find_by_id_in_op(&mut op, self.config.credit_facility_id)
            .await?;

        if credit_facility
            .acknowledge_payment_from_liquidation(self.config.liquidation_id)?
            .did_execute()
        {
            self.credit_facility_repo
                .update_in_op(&mut op, &mut credit_facility)
                .await?;
        }

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
