use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveDate, Utc};
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

use crate::config::CustomerActivityCheckConfig;
use crate::error::CustomerActivityError;
use job::*;

// Use January 1st, 2000 as the minimum date
const EARLIEST_SEARCH_START: DateTime<Utc> = {
    let date = NaiveDate::from_ymd_opt(2000, 1, 1)
        .expect("valid date")
        .and_hms_opt(0, 0, 0)
        .expect("valid time");
    DateTime::from_naive_utc_and_offset(date, Utc)
};

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

impl<Perms, E> Default for CustomerActivityCheckJobConfig<Perms, E> {
    fn default() -> Self {
        Self::new()
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
    config: CustomerActivityCheckConfig,
}

impl<Perms, E> CustomerActivityCheckInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<LanaEvent>
        + OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(customers: &Customers<Perms, E>, config: CustomerActivityCheckConfig) -> Self {
        Self {
            customers: customers.clone(),
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
    config: CustomerActivityCheckConfig,
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
        if self.config.activity_check_enabled {
            self.perform_activity_check().await?;
        }
        let (hours, minutes) = self.config.parse_activity_check_time()?;
        let next_run = calculate_next_run_time(now, hours, minutes)?;
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
        self.update_customers_by_activity_and_date_range(
            EARLIEST_SEARCH_START,
            self.config.get_escheatment_threshold_date(),
            core_customer::Activity::Suspended,
        )
        .await?;

        self.update_customers_by_activity_and_date_range(
            self.config.get_escheatment_threshold_date(),
            self.config.get_inactive_threshold_date(),
            core_customer::Activity::Inactive,
        )
        .await?;

        self.update_customers_by_activity_and_date_range(
            self.config.get_inactive_threshold_date(),
            crate::time::now(),
            core_customer::Activity::Active,
        )
        .await?;

        Ok(())
    }

    async fn update_customers_by_activity_and_date_range(
        &self,
        start_threshold: DateTime<Utc>,
        end_threshold: DateTime<Utc>,
        activity: core_customer::Activity,
    ) -> Result<(), CustomerActivityError> {
        let customers = self
            .customers
            .find_customers_with_activity_mismatch(start_threshold, end_threshold, activity)
            .await?;
        // TODO: Add a batch update for the customers
        for customer_id in customers {
            self.customers
                .update_activity_from_system(customer_id, activity)
                .await?;
        }

        Ok(())
    }
}

fn calculate_next_run_time(
    from_time: DateTime<Utc>,
    hours: u32,
    minutes: u32,
) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    let tomorrow = from_time + Duration::days(1);

    let midnight = tomorrow
        .date_naive()
        .and_hms_opt(hours, minutes, 0)
        .ok_or("Failed to create midnight time")?;

    let utc_midnight = midnight
        .and_local_timezone(Utc)
        .single()
        .ok_or("Failed to convert midnight to UTC timezone")?;

    Ok(utc_midnight)
}
