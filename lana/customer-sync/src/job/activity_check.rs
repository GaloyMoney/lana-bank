use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use tracing::instrument;

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use core_deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction,
    GovernanceObject,
};

use governance::GovernanceEvent;
use outbox::OutboxEventMarker;

use job::*;

use crate::config::*;

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
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    type Initializer = CustomerActivityCheckInit<Perms, E>;
}

pub struct CustomerActivityCheckInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    customers: Customers<Perms, E>,
    deposit: CoreDeposit<Perms, E>,
    config: CustomerSyncConfig,
}

impl<Perms, E> CustomerActivityCheckInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(
        customers: &Customers<Perms, E>,
        deposit: &CoreDeposit<Perms, E>,
        config: CustomerSyncConfig,
    ) -> Self {
        Self {
            customers: customers.clone(),
            deposit: deposit.clone(),
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
    E: OutboxEventMarker<CoreCustomerEvent>
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
            deposit: self.deposit.clone(),
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
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    customers: Customers<Perms, E>,
    deposit: CoreDeposit<Perms, E>,
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
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    #[instrument(name = "customer_activity_check.run", skip(self, _current_job), err)]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let now = Utc::now();

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
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    #[instrument(name = "customer_activity_check.perform_check", skip(self), err)]
    async fn perform_activity_check(&self) -> Result<(), Box<dyn std::error::Error>> {
        let now = Utc::now();
        let inactive_threshold = now - Duration::days(self.config.inactive_threshold_days);
        let escheatment_threshold = now - Duration::days(self.config.escheatment_threshold_days);

        let customers = self.customers.list_all_customers().await?;

        for customer in customers {
            let last_transaction_date = self.get_last_transaction_date(customer.id).await?;

            let new_activity = match last_transaction_date {
                Some(date) if date < escheatment_threshold => {
                    core_customer::AccountActivity::Suspended
                }
                Some(date) if date < inactive_threshold => core_customer::AccountActivity::Disabled,
                Some(_) => core_customer::AccountActivity::Enabled,
                None => core_customer::AccountActivity::Disabled,
            };

            if customer.activity != new_activity {
                self.customers
                    .update_account_activity_from_system(customer.id, new_activity)
                    .await?;
            }
        }

        Ok(())
    }

    async fn get_last_transaction_date(
        &self,
        customer_id: core_customer::CustomerId,
    ) -> Result<Option<DateTime<Utc>>, Box<dyn std::error::Error>> {
        let mut latest_date: Option<DateTime<Utc>> = None;
        let mut next = Some(es_entity::PaginatedQueryArgs {
            first: 100,
            after: None,
        });

        while let Some(query) = next.take() {
            let deposit_accounts = self.deposit
                .list_accounts_by_created_at_for_account_holder(
                    &<<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject as SystemSubject>::system(),
                    customer_id,
                    query,
                    es_entity::ListDirection::Descending,
                )
                .await?;

            if deposit_accounts.entities.is_empty() {
                break;
            }

            for deposit_account in &deposit_accounts.entities {
                let history = self.deposit
                    .account_history(
                        &<<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject as SystemSubject>::system(),
                        deposit_account.id,
                        es_entity::PaginatedQueryArgs { first: 1, after: None },
                    )
                    .await?;

                if let Some(entry) = history.entities.first() {
                    let transaction_date = match entry {
                        core_deposit::DepositAccountHistoryEntry::Deposit(entry) => {
                            entry.recorded_at
                        }
                        core_deposit::DepositAccountHistoryEntry::Withdrawal(entry) => {
                            entry.recorded_at
                        }
                        core_deposit::DepositAccountHistoryEntry::CancelledWithdrawal(entry) => {
                            entry.recorded_at
                        }
                        core_deposit::DepositAccountHistoryEntry::Disbursal(entry) => {
                            entry.recorded_at
                        }
                        core_deposit::DepositAccountHistoryEntry::Payment(entry) => {
                            entry.recorded_at
                        }
                        core_deposit::DepositAccountHistoryEntry::Unknown(entry) => {
                            entry.recorded_at
                        }
                        core_deposit::DepositAccountHistoryEntry::Ignored => continue,
                    };

                    latest_date = match latest_date {
                        Some(current_latest) if transaction_date > current_latest => {
                            Some(transaction_date)
                        }
                        Some(current_latest) => Some(current_latest),
                        None => Some(transaction_date),
                    };
                }
            }

            next = deposit_accounts.into_next_query();
        }

        Ok(latest_date)
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
