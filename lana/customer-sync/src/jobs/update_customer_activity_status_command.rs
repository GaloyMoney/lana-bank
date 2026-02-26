use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use job::*;
use obix::out::OutboxEventMarker;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{CoreDepositAction, CoreDepositObject, GovernanceAction, GovernanceObject};
use tracing_macros::record_error_severity;

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateCustomerActivityStatusConfig {
    pub closing_time: DateTime<Utc>,
}

pub const UPDATE_CUSTOMER_ACTIVITY_STATUS_COMMAND: JobType =
    JobType::new("command.customer-sync.update-customer-activity-status");

pub struct UpdateCustomerActivityStatusJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    customers: Customers<Perms, E>,
}

impl<Perms, E> UpdateCustomerActivityStatusJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pub fn new(customers: Customers<Perms, E>) -> Self {
        Self { customers }
    }
}

impl<Perms, E> JobInitializer for UpdateCustomerActivityStatusJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    type Config = UpdateCustomerActivityStatusConfig;

    fn job_type(&self) -> JobType {
        UPDATE_CUSTOMER_ACTIVITY_STATUS_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdateCustomerActivityStatusJobRunner {
            config: job.config()?,
            customers: self.customers.clone(),
        }))
    }
}

pub struct UpdateCustomerActivityStatusJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    config: UpdateCustomerActivityStatusConfig,
    customers: Customers<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for UpdateCustomerActivityStatusJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[record_error_severity]
    #[tracing::instrument(
        name = "customer_sync.update_customer_activity_status_job.process_command",
        skip(self, _current_job),
        fields(closing_time = %self.config.closing_time),
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.customers
            .perform_customer_activity_status_update(self.config.closing_time)
            .await?;
        Ok(JobCompletion::Complete)
    }
}
