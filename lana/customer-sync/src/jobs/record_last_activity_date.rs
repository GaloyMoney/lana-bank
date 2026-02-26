use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use job::*;
use obix::out::OutboxEventMarker;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, DepositAccountId,
    GovernanceAction, GovernanceObject,
};
use governance::GovernanceEvent;
use tracing_macros::record_error_severity;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecordLastActivityDateConfig {
    pub deposit_account_id: DepositAccountId,
    pub recorded_at: DateTime<Utc>,
    pub trace_context: tracing_utils::persistence::SerializableTraceContext,
}

pub const RECORD_LAST_ACTIVITY_DATE_COMMAND: JobType =
    JobType::new("command.customer-sync.record-last-activity-date");

pub struct RecordLastActivityDateJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    customers: Customers<Perms, E>,
    deposits: CoreDeposit<Perms, E>,
}

impl<Perms, E> RecordLastActivityDateJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(customers: Customers<Perms, E>, deposits: CoreDeposit<Perms, E>) -> Self {
        Self {
            customers,
            deposits,
        }
    }
}

impl<Perms, E> JobInitializer for RecordLastActivityDateJobInitializer<Perms, E>
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
    type Config = RecordLastActivityDateConfig;

    fn job_type(&self) -> JobType {
        RECORD_LAST_ACTIVITY_DATE_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(RecordLastActivityDateJobRunner {
            config: job.config()?,
            customers: self.customers.clone(),
            deposits: self.deposits.clone(),
        }))
    }
}

pub struct RecordLastActivityDateJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    config: RecordLastActivityDateConfig,
    customers: Customers<Perms, E>,
    deposits: CoreDeposit<Perms, E>,
}

#[async_trait]
impl<Perms, E> JobRunner for RecordLastActivityDateJobRunner<Perms, E>
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
        name = "customer_sync.record_last_activity_date_job.process_command",
        skip(self, _current_job),
        fields(deposit_account_id = %self.config.deposit_account_id),
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        tracing_utils::persistence::set_parent(&self.config.trace_context);
        let account = self
            .deposits
            .find_account_by_id_without_audit(self.config.deposit_account_id)
            .await?;

        let customer_id = account.account_holder_id.into();

        self.customers
            .record_last_activity_date(customer_id, self.config.recorded_at)
            .await?;

        Ok(JobCompletion::Complete)
    }
}
