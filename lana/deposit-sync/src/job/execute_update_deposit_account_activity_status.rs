use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction,
    GovernanceObject,
};
use core_time_events::CoreTimeEvent;
use governance::GovernanceEvent;
use job::*;
use lana_events::LanaEvent;
use obix::out::OutboxEventMarker;
use tracing_macros::record_error_severity;

pub const UPDATE_DEPOSIT_ACCOUNT_ACTIVITY_STATUS_COMMAND: JobType =
    JobType::new("command.deposit-sync.update-deposit-account-activity-status");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDepositAccountActivityStatusConfig {
    pub closing_time: DateTime<Utc>,
}

pub struct UpdateDepositAccountActivityStatusJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    deposits: CoreDeposit<Perms, E>,
}

impl<Perms, E> UpdateDepositAccountActivityStatusJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    pub fn new(deposits: &CoreDeposit<Perms, E>) -> Self {
        Self {
            deposits: deposits.clone(),
        }
    }
}

impl<Perms, E> JobInitializer for UpdateDepositAccountActivityStatusJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    type Config = UpdateDepositAccountActivityStatusConfig;

    fn job_type(&self) -> JobType {
        UPDATE_DEPOSIT_ACCOUNT_ACTIVITY_STATUS_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdateDepositAccountActivityStatusJobRunner {
            config: job.config()?,
            deposits: self.deposits.clone(),
        }))
    }
}

struct UpdateDepositAccountActivityStatusJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    config: UpdateDepositAccountActivityStatusConfig,
    deposits: CoreDeposit<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for UpdateDepositAccountActivityStatusJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>
        + OutboxEventMarker<CoreTimeEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "deposit_sync.update_deposit_account_activity_status.process_command",
        skip_all
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.deposits
            .perform_activity_status_update(self.config.closing_time)
            .await?;
        Ok(JobCompletion::Complete)
    }
}
