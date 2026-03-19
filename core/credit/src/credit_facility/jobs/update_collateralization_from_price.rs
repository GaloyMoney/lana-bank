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
use core_price::CorePriceEvent;
use governance::GovernanceEvent;
use job::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditCollectionEvent, CoreCreditEvent,
    credit_facility::{
        CreditFacilitiesByCollateralizationRatioCursor, CreditFacilityRepo, CreditFacilityStatus,
    },
    ledger::*,
    primitives::*,
};

pub const PRICE_SWEEP_COMMAND: JobType =
    JobType::new("command.credit.update-collateralization-from-price");

pub const PRICE_SWEEP_QUEUE_ID: &str = "credit-facility-price-sweep";

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCollateralizationFromPriceConfig {
    pub price: PriceOfOneBTC,
}

#[derive(Default, Serialize, Deserialize)]
struct UpdateCollateralizationFromPriceState {
    after: Option<CreditFacilitiesByCollateralizationRatioCursor>,
}

pub struct UpdateCollateralizationFromPriceJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    repo: Arc<CreditFacilityRepo<E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    ledger: Arc<CreditLedger>,
    authz: Arc<Perms>,
}

impl<Perms, E> UpdateCollateralizationFromPriceJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(
        repo: Arc<CreditFacilityRepo<E>>,
        collaterals: Arc<Collaterals<Perms, E>>,
        ledger: Arc<CreditLedger>,
        authz: Arc<Perms>,
    ) -> Self {
        Self {
            repo,
            collaterals,
            ledger,
            authz,
        }
    }
}

impl<Perms, E> JobInitializer for UpdateCollateralizationFromPriceJobInitializer<Perms, E>
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
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    type Config = UpdateCollateralizationFromPriceConfig;

    fn job_type(&self) -> JobType {
        PRICE_SWEEP_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdateCollateralizationFromPriceJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
            collaterals: self.collaterals.clone(),
            ledger: self.ledger.clone(),
            authz: self.authz.clone(),
        }))
    }
}

struct UpdateCollateralizationFromPriceJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: UpdateCollateralizationFromPriceConfig,
    repo: Arc<CreditFacilityRepo<E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    ledger: Arc<CreditLedger>,
    authz: Arc<Perms>,
}

#[async_trait]
impl<Perms, E> JobRunner for UpdateCollateralizationFromPriceJobRunner<Perms, E>
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
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "credit.update_collateralization_from_price.process_command",
        skip(self, current_job)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let price = self.config.price;
        let mut state = current_job
            .execution_state::<UpdateCollateralizationFromPriceState>()?
            .unwrap_or_default();

        loop {
            let credit_facilities = self
                .repo
                .list_by_collateralization_ratio(
                    es_entity::PaginatedQueryArgs {
                        first: 10,
                        after: state.after,
                    },
                    es_entity::ListDirection::Ascending,
                )
                .await?;

            let (next_cursor, has_next_page) = (
                credit_facilities.end_cursor,
                credit_facilities.has_next_page,
            );

            let mut op = current_job.begin_op().await?;
            self.authz
                .audit()
                .record_system_entry_in_op(
                    &mut op,
                    crate::primitives::COLLATERALIZATION_SYNC,
                    CoreCreditObject::all_credit_facilities(),
                    CoreCreditAction::CREDIT_FACILITY_UPDATE_COLLATERALIZATION_STATE,
                )
                .await?;

            let mut updated = Vec::new();
            for mut facility in credit_facilities.entities {
                if facility.status() == CreditFacilityStatus::Closed {
                    continue;
                }
                let collateral_account_id = self
                    .collaterals
                    .collateral_ledger_account_ids_in_op(&mut op, facility.collateral_id)
                    .await?
                    .collateral_account_id;
                let balances = self
                    .ledger
                    .get_credit_facility_balance_in_op(
                        &mut op,
                        facility.account_ids,
                        collateral_account_id,
                    )
                    .await?;
                if facility
                    .update_collateralization(price, CVLPct::UPGRADE_BUFFER, balances)
                    .did_execute()
                {
                    updated.push(facility);
                }
            }

            let n = self.repo.update_all_in_op(&mut op, &mut updated).await?;

            if n > 0 {
                state.after = next_cursor;
                current_job
                    .update_execution_state_in_op(&mut op, &state)
                    .await?;
                op.commit().await?;
            } else {
                break;
            }

            if !has_next_page {
                break;
            }
        }

        Ok(JobCompletion::Complete)
    }
}
