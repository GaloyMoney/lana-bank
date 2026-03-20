use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_credit_collateral::{
    Collaterals, CoreCreditCollateralAction, CoreCreditCollateralObject,
    public::CoreCreditCollateralEvent,
};
use core_credit_collection::{CoreCreditCollectionAction, CoreCreditCollectionObject};
use core_custody::CoreCustodyEvent;
use core_price::Price;
use governance::GovernanceEvent;
use job::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditCollectionEvent, CoreCreditEvent, credit_facility::CreditFacilityRepo, ledger::*,
    primitives::*,
};

pub const UPDATE_COLLATERALIZATION_COMMAND: JobType =
    JobType::new("command.credit.update-collateralization");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCollateralizationConfig {
    pub credit_facility_id: CreditFacilityId,
}

pub struct UpdateCollateralizationJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    repo: Arc<CreditFacilityRepo<E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    price: Arc<Price>,
    ledger: Arc<CreditLedger>,
    authz: Arc<Perms>,
}

impl<Perms, E> UpdateCollateralizationJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    pub fn new(
        repo: Arc<CreditFacilityRepo<E>>,
        collaterals: Arc<Collaterals<Perms, E>>,
        price: Arc<Price>,
        ledger: Arc<CreditLedger>,
        authz: Arc<Perms>,
    ) -> Self {
        Self {
            repo,
            collaterals,
            price,
            ledger,
            authz,
        }
    }
}

impl<Perms, E> JobInitializer for UpdateCollateralizationJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    type Config = UpdateCollateralizationConfig;

    fn job_type(&self) -> JobType {
        UPDATE_COLLATERALIZATION_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdateCollateralizationJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
            collaterals: self.collaterals.clone(),
            price: self.price.clone(),
            ledger: self.ledger.clone(),
            authz: self.authz.clone(),
        }))
    }
}

struct UpdateCollateralizationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    config: UpdateCollateralizationConfig,
    repo: Arc<CreditFacilityRepo<E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    price: Arc<Price>,
    ledger: Arc<CreditLedger>,
    authz: Arc<Perms>,
}

#[async_trait]
impl<Perms, E> JobRunner for UpdateCollateralizationJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<CoreCreditCollectionAction>
        + From<CoreCreditCollateralAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<CoreCreditCollectionObject>
        + From<CoreCreditCollateralObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "credit.update_collateralization.process_command",
        skip(self, current_job),
        fields(credit_facility_id = %self.config.credit_facility_id),
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        // if the pending facility is not collateralized enough to be activated there will be no
        // credit facility to update the collateralization state for
        let Some(mut credit_facility) = self
            .repo
            .maybe_find_by_id_in_op(&mut op, self.config.credit_facility_id)
            .await?
        else {
            return Ok(JobCompletion::CompleteWithOp(op));
        };

        self.authz
            .audit()
            .record_system_entry_in_op(
                &mut op,
                crate::primitives::COLLATERALIZATION_SYNC,
                CoreCreditObject::all_credit_facilities(),
                CoreCreditAction::CREDIT_FACILITY_UPDATE_COLLATERALIZATION_STATE,
            )
            .await?;

        let collateral_account_id = self
            .collaterals
            .collateral_ledger_account_ids_in_op(&mut op, credit_facility.collateral_id)
            .await?
            .collateral_account_id;

        let balances = self
            .ledger
            .get_credit_facility_balance_in_op(
                &mut op,
                credit_facility.account_ids,
                collateral_account_id,
            )
            .await?;

        let price = self.price.usd_cents_per_btc().await;

        if credit_facility
            .update_collateralization(price, CVLPct::UPGRADE_BUFFER, balances)
            .did_execute()
        {
            self.repo
                .update_in_op(&mut op, &mut credit_facility)
                .await?;
        }

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
