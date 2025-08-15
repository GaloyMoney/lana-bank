use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{
    CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction, GovernanceObject,
};

use es_entity::prelude::sqlx;
use governance::GovernanceEvent;
use lana_events::LanaEvent;
use outbox::OutboxEventMarker;

use job::*;

use crate::config::*;
use customer_activity::CustomerActivityService;

#[derive(serde::Serialize)]
pub struct CustomerActivityCheckJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> CustomerActivityCheckJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> JobConfig for CustomerActivityCheckJobConfig<Perms, E>
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
    type Initializer = CustomerActivityCheckInit<Perms, E>;
}

pub struct CustomerActivityCheckInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    customers: Customers<Perms, E>,
    customer_activity: CustomerActivityService,
    config: CustomerSyncConfig,
}

impl<Perms, E> CustomerActivityCheckInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(
        customers: &Customers<Perms, E>,
        pool: sqlx::PgPool,
        config: CustomerSyncConfig,
    ) -> Self {
        Self {
            customers: customers.clone(),
            customer_activity: CustomerActivityService::new(pool),
            config,
        }
    }
}

const CUSTOMER_ACTIVITY_CHECK: JobType = JobType::new("customer-activity-check");

impl<Perms, E> JobInitializer for CustomerActivityCheckInit<Perms, E>
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
        CUSTOMER_ACTIVITY_CHECK
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CustomerActivityCheckJobRunner {
            customers: self.customers.clone(),
            customer_activity: self.customer_activity.clone(),
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

pub struct CustomerActivityCheckJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    customers: Customers<Perms, E>,
    customer_activity: CustomerActivityService,
    config: CustomerSyncConfig,
}

#[async_trait]
impl<Perms, E> JobRunner for CustomerActivityCheckJobRunner<Perms, E>
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
    #[instrument(name = "customer_activity_check.run", skip(self, _current_job), err)]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let now = crate::time::now();

        if !self.config.activity_check_enabled {
            let next_run = calculate_next_run_time(
                now,
                self.config.activity_check_hour,
                self.config.activity_check_minute,
            );
            return Ok(JobCompletion::RescheduleAt(next_run));
        }

        self.perform_activity_check().await?;

        let next_run = calculate_next_run_time(
            now,
            self.config.activity_check_hour,
            self.config.activity_check_minute,
        );
        Ok(JobCompletion::RescheduleAt(next_run))
    }
}

impl<Perms, E> CustomerActivityCheckJobRunner<Perms, E>
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
    #[instrument(name = "customer_activity_check.perform_check", skip(self), err)]
    async fn perform_activity_check(&self) -> Result<(), Box<dyn std::error::Error>> {
        let now = crate::time::now();
        let inactive_threshold = now - Duration::days(self.config.inactive_threshold_days);
        let escheatment_threshold = now - Duration::days(self.config.escheatment_threshold_days);
        let min_date = NaiveDate::MIN.and_hms_opt(0, 0, 0).unwrap().and_utc();

        self.update_customers_by_activity_range(
            min_date,
            escheatment_threshold,
            core_customer::Activity::Suspended,
        )
        .await?;

        self.update_customers_by_activity_range(
            escheatment_threshold,
            inactive_threshold,
            core_customer::Activity::Disabled,
        )
        .await?;

        self.update_customers_by_activity_range(
            inactive_threshold,
            now,
            core_customer::Activity::Enabled,
        )
        .await?;

        self.update_customers_without_activity().await?;

        Ok(())
    }

    async fn update_customers_by_activity_range(
        &self,
        start_threshold: DateTime<Utc>,
        end_threshold: DateTime<Utc>,
        activity: core_customer::Activity,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let customers = self
            .customer_activity
            .find_customers_with_activity_in_range(start_threshold, end_threshold)
            .await?;

        for customer_id in customers {
            self.customers
                .update_activity_from_system(customer_id, activity)
                .await?;
        }

        Ok(())
    }

    async fn update_customers_without_activity(&self) -> Result<(), Box<dyn std::error::Error>> {
        let customers = self
            .customer_activity
            .find_customers_without_activity()
            .await?;

        for customer_id in customers {
            self.customers
                .update_activity_from_system(customer_id, core_customer::Activity::Disabled)
                .await?;
        }

        Ok(())
    }
}

fn calculate_next_run_time(from_time: DateTime<Utc>, hour: u32, minute: u32) -> DateTime<Utc> {
    let tomorrow = from_time + Duration::days(1);
    tomorrow
        .date_naive()
        .and_hms_opt(hour, minute, 0)
        .unwrap()
        .and_local_timezone(Utc)
        .unwrap()
}
