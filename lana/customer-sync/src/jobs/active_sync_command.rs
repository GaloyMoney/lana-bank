use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;
use obix::out::OutboxEventMarker;

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use core_customer::{
    CUSTOMER_SYNC, CoreCustomerAction, CoreCustomerEvent, CustomerId, CustomerObject,
};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject,
    DepositAccountHolderStatus, GovernanceAction, GovernanceObject,
};
use governance::GovernanceEvent;
use tracing_macros::record_error_severity;

#[derive(Serialize, Deserialize, Clone)]
pub struct CustomerActiveSyncConfig {
    pub customer_id: CustomerId,
}

pub const CUSTOMER_ACTIVE_SYNC_COMMAND: JobType =
    JobType::new("command.customer-sync.customer-active-sync");

pub struct CustomerActiveSyncCommandInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    deposit: CoreDeposit<Perms, E>,
}

impl<Perms, E> CustomerActiveSyncCommandInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(deposit: CoreDeposit<Perms, E>) -> Self {
        Self { deposit }
    }
}

impl<Perms, E> JobInitializer for CustomerActiveSyncCommandInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    type Config = CustomerActiveSyncConfig;

    fn job_type(&self) -> JobType {
        CUSTOMER_ACTIVE_SYNC_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CustomerActiveSyncCommandRunner {
            config: job.config()?,
            deposit: self.deposit.clone(),
        }))
    }
}

pub struct CustomerActiveSyncCommandRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    config: CustomerActiveSyncConfig,
    deposit: CoreDeposit<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for CustomerActiveSyncCommandRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "customer_sync.customer_active_sync_command.run",
        skip(self, _current_job),
        fields(customer_id = %self.config.customer_id),
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.deposit
            .update_account_status_for_holder(
                &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(CUSTOMER_SYNC),
                self.config.customer_id,
                DepositAccountHolderStatus::Active,
            )
            .await?;
        Ok(JobCompletion::Complete)
    }
}
