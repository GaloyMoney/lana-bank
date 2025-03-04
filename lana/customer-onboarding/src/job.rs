use applicant::Applicants;
use async_trait::async_trait;
use futures::StreamExt;
use kratos_admin::KratosAdmin;
use tracing::instrument;

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use core_customer::{
    AuthenticationId, CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers,
};
use deposit::{
    CoreDeposit, CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction,
    GovernanceObject,
};
use governance::GovernanceEvent;
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use job::*;

use crate::config::*;

#[derive(serde::Serialize)]
pub struct CustomerOnboardingJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> CustomerOnboardingJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<Perms, E> JobConfig for CustomerOnboardingJobConfig<Perms, E>
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
    type Initializer = CustomerOnboardingJobInitializer<Perms, E>;
}

pub struct CustomerOnboardingJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    kratos_admin: KratosAdmin,
    customers: Customers<Perms, E>,
    deposit: CoreDeposit<Perms, E>,
    applicants: Applicants<Perms, E>,
    config: CustomerOnboardingConfig,
}

impl<Perms, E> CustomerOnboardingJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        customers: &Customers<Perms, E>,
        deposit: &CoreDeposit<Perms, E>,
        applicant: &Applicants<Perms, E>,
        config: CustomerOnboardingConfig,
    ) -> Self {
        let kratos_admin = kratos_admin::KratosAdmin::init(config.kratos_admin.clone());

        Self {
            outbox: outbox.clone(),
            customers: customers.clone(),
            deposit: deposit.clone(),
            applicants: applicant.clone(),
            kratos_admin,
            config,
        }
    }
}

const CUSTOMER_ONBOARDING_JOB: JobType = JobType::new("customer-onboarding");
impl<Perms, E> JobInitializer for CustomerOnboardingJobInitializer<Perms, E>
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
        CUSTOMER_ONBOARDING_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CustomerOnboardingJobRunner {
            outbox: self.outbox.clone(),
            customers: self.customers.clone(),
            deposit: self.deposit.clone(),
            applicants: self.applicants.clone(),
            kratos_admin: self.kratos_admin.clone(),
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

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
struct CustomerOnboardingJobData {
    sequence: outbox::EventSequence,
}

pub struct CustomerOnboardingJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>
        + OutboxEventMarker<CoreDepositEvent>
        + OutboxEventMarker<GovernanceEvent>,
{
    outbox: Outbox<E>,
    customers: Customers<Perms, E>,
    deposit: CoreDeposit<Perms, E>,
    applicants: Applicants<Perms, E>,
    kratos_admin: KratosAdmin,
    config: CustomerOnboardingConfig,
}
#[async_trait]
impl<Perms, E> JobRunner for CustomerOnboardingJobRunner<Perms, E>
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
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let state = current_job
            .execution_state::<CustomerOnboardingJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            if let Some(CoreCustomerEvent::CustomerCreated { .. }) = &message.as_ref().as_event() {
                self.handle_customer_created_event(message.as_ref()).await?;
            }
        }

        let now = crate::time::now();
        Ok(JobCompletion::RescheduleAt(now))
    }
}

impl<Perms, E> CustomerOnboardingJobRunner<Perms, E>
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
    #[instrument(
        name = "customer_onboarding.handle_customer_created_event",
        skip(self, message)
    )]
    async fn handle_customer_created_event(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        E: OutboxEventMarker<CoreCustomerEvent>,
    {
        if let Some(CoreCustomerEvent::CustomerCreated { id, email }) = message.as_event() {
            message.inject_trace_parent();

            if self.config.auto_create_deposit_account {
                let description = &format!("Deposit Account for Customer {}", id);
                let account_ref = &format!("deposit-customer-account:{}", id);
                match self.deposit
                .create_account(&<<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject as SystemSubject>::system(), *id, account_ref,
                "customer-deposits", description)
                .await {
                Ok(_) => {}
                Err(e) if e.is_account_already_exists() => {},
                Err(e) => return Err(e.into()),
                }
            }

            let authentication_id = self
                .kratos_admin
                .create_user::<AuthenticationId>(email.clone())
                .await?;
            self.customers
                .update_authentication_id_for_customer(*id, authentication_id)
                .await?;
            self.applicants.create_permalink(*id).await?;
        }
        Ok(())
    }
}
