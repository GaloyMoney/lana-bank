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
use core_price::CorePriceEvent;
use governance::GovernanceEvent;
use job::*;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditEvent,
    ledger::*,
    pending_credit_facility::{
        PendingCreditFacilitiesByCollateralizationRatioCursor, PendingCreditFacilityRepo,
    },
    primitives::*,
};

pub const PENDING_PRICE_SWEEP_COMMAND: JobType =
    JobType::new("command.credit.update-pending-collateralization-from-price");

pub const PENDING_PRICE_SWEEP_QUEUE_ID: &str = "pending-credit-facility-price-sweep";

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePendingCollateralizationFromPriceConfig {
    pub price: PriceOfOneBTC,
}

#[derive(Default, Serialize, Deserialize)]
struct UpdatePendingCollateralizationFromPriceState {
    after: Option<PendingCreditFacilitiesByCollateralizationRatioCursor>,
}

pub struct UpdatePendingCollateralizationFromPriceJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    repo: Arc<PendingCreditFacilityRepo<E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    ledger: Arc<CreditLedger>,
}

impl<Perms, E> UpdatePendingCollateralizationFromPriceJobInitializer<Perms, E>
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
        repo: Arc<PendingCreditFacilityRepo<E>>,
        collaterals: Arc<Collaterals<Perms, E>>,
        ledger: Arc<CreditLedger>,
    ) -> Self {
        Self {
            repo,
            collaterals,
            ledger,
        }
    }
}

impl<Perms, E> JobInitializer for UpdatePendingCollateralizationFromPriceJobInitializer<Perms, E>
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
    type Config = UpdatePendingCollateralizationFromPriceConfig;

    fn job_type(&self) -> JobType {
        PENDING_PRICE_SWEEP_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdatePendingCollateralizationFromPriceJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
            collaterals: self.collaterals.clone(),
            ledger: self.ledger.clone(),
        }))
    }
}

struct UpdatePendingCollateralizationFromPriceJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: UpdatePendingCollateralizationFromPriceConfig,
    repo: Arc<PendingCreditFacilityRepo<E>>,
    collaterals: Arc<Collaterals<Perms, E>>,
    ledger: Arc<CreditLedger>,
}

#[async_trait]
impl<Perms, E> JobRunner for UpdatePendingCollateralizationFromPriceJobRunner<Perms, E>
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
        name = "credit.update_pending_collateralization_from_price.process_command",
        skip(self, current_job)
    )]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let price = self.config.price;
        let mut state = current_job
            .execution_state::<UpdatePendingCollateralizationFromPriceState>()?
            .unwrap_or_default();

        loop {
            let pending_credit_facilities = self
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
                pending_credit_facilities.end_cursor,
                pending_credit_facilities.has_next_page,
            );

            let mut op = current_job.begin_op().await?;

            let mut updated = Vec::new();
            for mut pending_facility in pending_credit_facilities.entities {
                if pending_facility.status() == PendingCreditFacilityStatus::Completed {
                    continue;
                }
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
                if pending_facility
                    .update_collateralization(price, balances)
                    .did_execute()
                {
                    updated.push(pending_facility);
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
