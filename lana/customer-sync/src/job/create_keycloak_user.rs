use async_trait::async_trait;
use futures::StreamExt;
use keycloak_admin::KeycloakAdmin;
use tracing::instrument;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_customer::{
    AuthenticationId, CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers,
};
use core_deposit::{
    CoreDepositAction, CoreDepositEvent, CoreDepositObject, GovernanceAction, GovernanceObject,
};
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use job::*;

#[derive(serde::Serialize)]
pub struct CreateKeycloakUserJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}
impl<Perms, E> CreateKeycloakUserJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<Perms, E> JobConfig for CreateKeycloakUserJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    type Initializer = CreateKeycloakUserInit<Perms, E>;
}

pub struct CreateKeycloakUserInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    outbox: Outbox<E>,
    keycloak_admin: KeycloakAdmin,
    customers: Customers<Perms, E>,
}

impl<Perms, E> CreateKeycloakUserInit<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        customers: &Customers<Perms, E>,
        keycloak_admin: KeycloakAdmin,
    ) -> Self {
        Self {
            outbox: outbox.clone(),
            customers: customers.clone(),
            keycloak_admin,
        }
    }
}

const CUSTOMER_SYNC_CREATE_KEYCLOAK_USER: JobType =
    JobType::new("customer-sync-create-keycloak-user");
impl<Perms, E> JobInitializer for CreateKeycloakUserInit<Perms, E>
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
        CUSTOMER_SYNC_CREATE_KEYCLOAK_USER
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreateKeycloakUserJobRunner {
            outbox: self.outbox.clone(),
            customers: self.customers.clone(),
            keycloak_admin: self.keycloak_admin.clone(),
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
struct CreateKeycloakUserJobData {
    sequence: outbox::EventSequence,
}

pub struct CreateKeycloakUserJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    outbox: Outbox<E>,
    customers: Customers<Perms, E>,
    keycloak_admin: KeycloakAdmin,
}
#[async_trait]
impl<Perms, E> JobRunner for CreateKeycloakUserJobRunner<Perms, E>
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
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CreateKeycloakUserJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            if let Some(CoreCustomerEvent::CustomerCreated { .. }) = &message.as_ref().as_event() {
                self.handle_create_keycloak_user(message.as_ref()).await?;
            }

            state.sequence = message.sequence;
            current_job.update_execution_state(state.clone()).await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

impl<Perms, E> CreateKeycloakUserJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<CoreCustomerAction> + From<CoreDepositAction> + From<GovernanceAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<CustomerObject> + From<CoreDepositObject> + From<GovernanceObject>,
    E: OutboxEventMarker<CoreCustomerEvent> + OutboxEventMarker<CoreDepositEvent>,
{
    #[instrument(name = "customer_sync.create_keycloak_user", skip(self, message))]
    async fn handle_create_keycloak_user(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        E: OutboxEventMarker<CoreCustomerEvent>,
    {
        if let Some(CoreCustomerEvent::CustomerCreated { id, email, .. }) = message.as_event() {
            message.inject_trace_parent();

            let uuid = self.keycloak_admin.create_user(email.clone()).await?;
            let authentication_id = AuthenticationId::from(uuid);
            self.customers
                .update_authentication_id_for_customer(*id, authentication_id)
                .await?;
        }
        Ok(())
    }
}
