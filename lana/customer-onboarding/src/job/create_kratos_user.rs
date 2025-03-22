use async_trait::async_trait;
use futures::StreamExt;
use kratos_admin::KratosAdmin;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{
    AuthenticationId, CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers,
};
use deposit::{
    CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction, GovernanceObject,
};
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use job::*;

use crate::config::*;

#[derive(serde::Serialize)]
pub struct CreateKratosUserJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> CreateKratosUserJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<Perms, E> JobConfig for CreateKratosUserJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    type Initializer = CreateKratosUserJobInitializer<Perms, E>;
}

pub struct CreateKratosUserJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    outbox: Outbox<E>,
    kratos_admin: KratosAdmin,
    customers: Customers<Perms, E>,
}

impl<Perms, E> CreateKratosUserJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        customers: &Customers<Perms, E>,
        config: CustomerOnboardingConfig,
    ) -> Self {
        let kratos_admin = kratos_admin::KratosAdmin::init(config.kratos_admin.clone());

        Self {
            outbox: outbox.clone(),
            customers: customers.clone(),
            kratos_admin,
        }
    }
}

const CUSTOMER_ONBOARDING_CREATE_KRATOS_USER: JobType =
    JobType::new("customer-onboarding-create-kratos-user");
impl<Perms, E> JobInitializer for CreateKratosUserJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CUSTOMER_ONBOARDING_CREATE_KRATOS_USER
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreateKratosUserJobRunner {
            outbox: self.outbox.clone(),
            customers: self.customers.clone(),
            kratos_admin: self.kratos_admin.clone(),
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
struct CreateKratosUserJobData {
    sequence: outbox::EventSequence,
}

pub struct CreateKratosUserJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    outbox: Outbox<E>,
    customers: Customers<Perms, E>,
    kratos_admin: KratosAdmin,
}
#[async_trait]
impl<Perms, E> JobRunner for CreateKratosUserJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let state = current_job
            .execution_state::<CreateKratosUserJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            if let Some(CoreCustomerEvent::CustomerCreated { .. }) = &message.as_ref().as_event() {
                self.handle_create_kratos_user(message.as_ref()).await?;
            }
        }

        let now = crate::time::now();
        Ok(JobCompletion::RescheduleAt(now))
    }
}

impl<Perms, E> CreateKratosUserJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    #[instrument(name = "customer_onboarding.create_kratos_user", skip(self, message))]
    async fn handle_create_kratos_user(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        E: OutboxEventMarker<CoreCustomerEvent>,
    {
        if let Some(CoreCustomerEvent::CustomerCreated { id, email, .. }) = message.as_event() {
            message.inject_trace_parent();

            let authentication_id = self
                .kratos_admin
                .create_user::<AuthenticationId>(email.clone())
                .await?;
            self.customers
                .update_authentication_id_for_customer(*id, authentication_id)
                .await?;
        }
        Ok(())
    }
}
