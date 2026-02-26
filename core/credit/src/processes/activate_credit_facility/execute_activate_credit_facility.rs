use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_custody::{CoreCustodyAction, CoreCustodyEvent, CoreCustodyObject};
use core_price::CorePriceEvent;
use governance::{GovernanceAction, GovernanceEvent, GovernanceObject};
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditAction, CoreCreditCollectionEvent, CoreCreditEvent, CoreCreditObject,
    collateral::{
        CoreCreditCollateralAction, CoreCreditCollateralObject, public::CoreCreditCollateralEvent,
    },
    primitives::PendingCreditFacilityId,
};

use super::ActivateCreditFacility;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteActivateCreditFacilityConfig {
    pub pending_credit_facility_id: PendingCreditFacilityId,
    #[serde(default)]
    pub trace_context: Option<tracing_utils::persistence::SerializableTraceContext>,
}

pub const EXECUTE_ACTIVATE_CREDIT_FACILITY_COMMAND: JobType =
    JobType::new("command.credit.execute-activate-credit-facility");

pub struct ExecuteActivateCreditFacilityJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    process: ActivateCreditFacility<Perms, E>,
}

impl<Perms, E> ExecuteActivateCreditFacilityJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    pub fn new(process: ActivateCreditFacility<Perms, E>) -> Self {
        Self { process }
    }
}

impl<Perms, E> JobInitializer for ExecuteActivateCreditFacilityJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<crate::CoreCreditCollectionAction>
        + From<CoreCreditCollateralAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<crate::CoreCreditCollectionObject>
        + From<CoreCreditCollateralObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    type Config = ExecuteActivateCreditFacilityConfig;

    fn job_type(&self) -> JobType {
        EXECUTE_ACTIVATE_CREDIT_FACILITY_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ExecuteActivateCreditFacilityJobRunner {
            config: job.config()?,
            process: self.process.clone(),
        }))
    }
}

pub struct ExecuteActivateCreditFacilityJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    config: ExecuteActivateCreditFacilityConfig,
    process: ActivateCreditFacility<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for ExecuteActivateCreditFacilityJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>
        + From<crate::CoreCreditCollectionAction>
        + From<CoreCreditCollateralAction>
        + From<GovernanceAction>
        + From<CoreCustodyAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>
        + From<crate::CoreCreditCollectionObject>
        + From<CoreCreditCollateralObject>
        + From<GovernanceObject>
        + From<CoreCustodyObject>,
    E: OutboxEventMarker<CoreCreditEvent>
        + OutboxEventMarker<CoreCreditCollateralEvent>
        + OutboxEventMarker<CoreCreditCollectionEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreCustodyEvent>
        + OutboxEventMarker<CorePriceEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "core_credit.execute_activate_credit_facility_job.process_command",
        skip(self, _current_job),
        fields(pending_credit_facility_id = %self.config.pending_credit_facility_id),
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        if let Some(ref ctx) = self.config.trace_context {
            tracing_utils::persistence::set_parent(ctx);
        }
        self.process
            .execute_activate_credit_facility(self.config.pending_credit_facility_id)
            .await?;

        Ok(JobCompletion::Complete)
    }
}
