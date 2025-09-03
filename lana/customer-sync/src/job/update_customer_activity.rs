use async_trait::async_trait;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{
    CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction, GovernanceObject,
};

use governance::GovernanceEvent;
use lana_events::LanaEvent;
use outbox::OutboxEventMarker;

use crate::config::CustomerSyncConfig;
use job::*;

#[derive(serde::Serialize)]
pub struct UpdateCustomerActivityJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> UpdateCustomerActivityJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> Default for UpdateCustomerActivityJobConfig<Perms, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Perms, E> JobConfig for UpdateCustomerActivityJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    type Initializer = UpdateCustomerActivityInit<Perms, E>;
}

pub struct UpdateCustomerActivityInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    customers: Customers<Perms, E>,
    config: CustomerSyncConfig,
}

impl<Perms, E> UpdateCustomerActivityInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(customers: &Customers<Perms, E>, config: CustomerSyncConfig) -> Self {
        Self {
            customers: customers.clone(),
            config,
        }
    }
}

const UPDATE_CUSTOMER_ACTIVITY: JobType = JobType::new("update-customer-activity");

impl<Perms, E> JobInitializer for UpdateCustomerActivityInit<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        UPDATE_CUSTOMER_ACTIVITY
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdateCustomerActivityJobRunner {
            customers: self.customers.clone(),
            config: self.config.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct UpdateCustomerActivityJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    customers: Customers<Perms, E>,
    config: CustomerSyncConfig,
}

#[async_trait]
impl<Perms, E> JobRunner for UpdateCustomerActivityJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    #[instrument(name = "update_customer_activity.run", skip(self, _current_job), err)]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let now = crate::time::now();
        if self.config.activity_update_enabled {
            self.customers.perform_activity_update().await?;
        }
        let next_run_at = self.config.activity_update_utc_time.next_after(now);
        Ok(JobCompletion::RescheduleAt(next_run_at))
    }
}
