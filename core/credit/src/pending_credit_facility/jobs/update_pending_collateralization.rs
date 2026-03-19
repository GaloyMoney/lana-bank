use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_credit_collateral::{
    Collaterals, CoreCreditCollateralAction, CoreCreditCollateralObject,
    public::CoreCreditCollateralEvent,
};
use core_credit_collection::{CoreCreditCollectionAction, CoreCreditCollectionEvent};
use core_custody::CoreCustodyEvent;
use core_price::Price;
use governance::GovernanceEvent;
use job::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditEvent, ledger::*, pending_credit_facility::PendingCreditFacilityRepo, primitives::*,
};

pub const UPDATE_PENDING_COLLATERALIZATION_COMMAND: JobType =
    JobType::new("command.credit.update-pending-collateralization");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePendingCollateralizationConfig {
    pub pending_credit_facility_id: PendingCreditFacilityId,
}

pub struct UpdatePendingCollateralizationJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    repo: Arc<PendingCreditFacilityRepo<E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    price: Arc<Price>,
    ledger: Arc<CreditLedger>,
}

impl<Perms, E> UpdatePendingCollateralizationJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        repo: Arc<PendingCreditFacilityRepo<E>>,
        collaterals: Arc<Collaterals<Perms, E>>,
        price: Arc<Price>,
        ledger: Arc<CreditLedger>,
    ) -> Self {
        Self {
            repo,
            collaterals,
            price,
            ledger,
        }
    }
}

impl<Perms, E> JobInitializer for UpdatePendingCollateralizationJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditCollectionAction> + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditCollectionObject> + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Config = UpdatePendingCollateralizationConfig;

    fn job_type(&self) -> JobType {
        UPDATE_PENDING_COLLATERALIZATION_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdatePendingCollateralizationJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
            collaterals: self.collaterals.clone(),
            price: self.price.clone(),
            ledger: self.ledger.clone(),
        }))
    }
}

struct UpdatePendingCollateralizationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    config: UpdatePendingCollateralizationConfig,
    repo: Arc<PendingCreditFacilityRepo<E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    price: Arc<Price>,
    ledger: Arc<CreditLedger>,
}

#[async_trait]
impl<Perms, E> JobRunner for UpdatePendingCollateralizationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCreditCollectionAction> + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CoreCreditCollectionObject> + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "credit.update_pending_collateralization.process_command",
        skip(self, current_job),
        fields(pending_credit_facility_id = %self.config.pending_credit_facility_id),
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        let mut pending_facility = self
            .repo
            .find_by_id_in_op(&mut op, self.config.pending_credit_facility_id)
            .await?;

        let collateral_account_id = self
            .collaterals
            .collateral_ledger_account_ids_in_op(&mut op, pending_facility.collateral_id)
            .await?
            .collateral_account_id;

        let balances = self
            .ledger
            .get_pending_credit_facility_balance_in_op(
                &mut op,
                pending_facility.account_ids,
                collateral_account_id,
            )
            .await?;

        let price = self.price.usd_cents_per_btc().await;

        if pending_facility
            .update_collateralization(price, balances)
            .did_execute()
        {
            self.repo
                .update_in_op(&mut op, &mut pending_facility)
                .await?;
        }

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
