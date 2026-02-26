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
#[serde(rename_all = "camelCase")]
pub struct PerformCustomerActivityStatusUpdateConfig {
    pub closing_time: DateTime<Utc>,
    pub trace_context: tracing_utils::persistence::SerializableTraceContext,
}

pub const PERFORM_CUSTOMER_ACTIVITY_STATUS_UPDATE_COMMAND: JobType =
    JobType::new("command.customer-sync.perform-customer-activity-status-update");

pub struct PerformCustomerActivityStatusUpdateJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    customers: Customers<Perms, E>,
}

impl<Perms, E> PerformCustomerActivityStatusUpdateJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pub fn new(customers: Customers<Perms, E>) -> Self {
        Self { customers }
    }
}

impl<Perms, E> JobInitializer for PerformCustomerActivityStatusUpdateJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    type Config = PerformCustomerActivityStatusUpdateConfig;

    fn job_type(&self) -> JobType {
        PERFORM_CUSTOMER_ACTIVITY_STATUS_UPDATE_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(PerformCustomerActivityStatusUpdateJobRunner {
            config: job.config()?,
            customers: self.customers.clone(),
        }))
    }
}

pub struct PerformCustomerActivityStatusUpdateJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    config: PerformCustomerActivityStatusUpdateConfig,
    customers: Customers<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for PerformCustomerActivityStatusUpdateJobRunner<Perms, E>
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
        name = "customer_sync.perform_customer_activity_status_update_job.process_command",
        skip(self, _current_job),
        fields(closing_time = %self.config.closing_time),
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        tracing_utils::persistence::set_parent(&self.config.trace_context);
        self.customers
            .perform_customer_activity_status_update(self.config.closing_time)
            .await?;
        Ok(JobCompletion::Complete)
    }
}
