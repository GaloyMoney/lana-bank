use async_trait::async_trait;
use futures::StreamExt;
use tracing::instrument;

use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use core_customer::{CoreCustomerAction, CoreCustomerEvent, CustomerObject, Customers};
use kratos_admin::KratosAdmin;
use outbox::{Outbox, OutboxEventMarker, PersistentOutboxEvent};

use job::error::JobRunError;
use job::*;

use crate::config::*;

#[derive(serde::Serialize)]
pub struct SyncEmailJobConfig<Perms, E> {
    _phantom: std::marker::PhantomData<(Perms, E)>,
}

impl<Perms, E> SyncEmailJobConfig<Perms, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Perms, E> JobConfig for SyncEmailJobConfig<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    type Initializer = SyncEmailJobInitializer<Perms, E>;
}

pub struct SyncEmailJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    outbox: Outbox<E>,
    customers: Customers<Perms, E>,
    kratos_admin: KratosAdmin,
}

impl<Perms, E> SyncEmailJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    pub fn new(
        outbox: &Outbox<E>,
        customers: &Customers<Perms, E>,
        config: CustomerSyncConfig,
    ) -> Self {
        let kratos_admin = kratos_admin::KratosAdmin::init(config.kratos_admin.clone());

        Self {
            outbox: outbox.clone(),
            customers: customers.clone(),
            kratos_admin,
        }
    }
}

const SYNC_EMAIL_JOB: JobType = JobType::new("sync-email-job");
impl<Perms, E> JobInitializer for SyncEmailJobInitializer<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        SYNC_EMAIL_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(SyncEmailJobRunner {
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
struct SyncEmailJobData {
    sequence: outbox::EventSequence,
}

pub struct SyncEmailJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    outbox: Outbox<E>,
    customers: Customers<Perms, E>,
    kratos_admin: KratosAdmin,
}

#[async_trait]
impl<Perms, E> JobRunner for SyncEmailJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    async fn run(&self, mut current_job: CurrentJob) -> Result<JobCompletion, JobRunError> {
        let mut state = current_job
            .execution_state::<SyncEmailJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            if let Some(CoreCustomerEvent::CustomerEmailUpdated { .. }) =
                &message.as_ref().as_event()
            {
                self.handle_email_update(message.as_ref()).await?;
                state.sequence = message.sequence;
                current_job.update_execution_state(&state).await?;
            }
        }

        Ok(JobCompletion::RescheduleNow)
    }
}

impl<Perms, E> SyncEmailJobRunner<Perms, E>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCustomerAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CustomerObject>,
    E: OutboxEventMarker<CoreCustomerEvent>,
{
    #[instrument(name = "customer_sync.sync_email", skip(self, message))]
    async fn handle_email_update(
        &self,
        message: &PersistentOutboxEvent<E>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        E: OutboxEventMarker<CoreCustomerEvent>,
    {
        if let Some(CoreCustomerEvent::CustomerEmailUpdated { id, email }) = message.as_event() {
            message.inject_trace_parent();

            let customer = self
                .customers
                .find_by_id(
                    &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system(),
                    *id,
                )
                .await?;

            if let Some(customer) = customer {
                if let Some(authentication_id) = customer.authentication_id {
                    self.kratos_admin
                        .update_user_email(authentication_id.into(), email.clone())
                        .await?;
                }
            }
        }
        Ok(())
    }
}
