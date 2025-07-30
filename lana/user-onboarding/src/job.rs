use async_trait::async_trait;
use core_access::{AuthenticationId, CoreAccessEvent};
use futures::StreamExt;

use job::*;

use audit::AuditSvc;
use core_access::{CoreAccessAction, CoreAccessObject, UserId, user::Users};
use outbox::{Outbox, OutboxEventMarker};

use kratos_admin::KratosAdmin;

#[derive(serde::Serialize)]
pub struct UserOnboardingJobConfig<Audit, E> {
    _phantom: std::marker::PhantomData<(Audit, E)>,
}
impl<Audit, E> UserOnboardingJobConfig<Audit, E> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
impl<Audit, E> JobConfig for UserOnboardingJobConfig<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    type Initializer = UserOnboardingInit<Audit, E>;
}

pub struct UserOnboardingInit<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    outbox: Outbox<E>,
    kratos_admin: KratosAdmin,
    users: Users<Audit, E>,
}

impl<Audit, E> UserOnboardingInit<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pub fn new(outbox: &Outbox<E>, users: &Users<Audit, E>, kratos_admin: KratosAdmin) -> Self {
        Self {
            outbox: outbox.clone(),
            users: users.clone(),
            kratos_admin,
        }
    }
}

const USER_ONBOARDING_JOB: JobType = JobType::new("user-onboarding");
impl<Audit, E> JobInitializer for UserOnboardingInit<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        USER_ONBOARDING_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UserOnboardingJobRunner {
            outbox: self.outbox.clone(),
            users: self.users.clone(),
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
struct UserOnboardingJobData {
    sequence: outbox::EventSequence,
}

pub struct UserOnboardingJobRunner<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    outbox: Outbox<E>,
    users: Users<Audit, E>,
    kratos_admin: KratosAdmin,
}
#[async_trait]
impl<Audit, E> JobRunner for UserOnboardingJobRunner<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<UserOnboardingJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            if let Some(CoreAccessEvent::UserCreated { id, email, .. }) =
                &message.as_ref().as_event()
            {
                let authentication_id = self
                    .kratos_admin
                    .create_user::<AuthenticationId>(email.clone())
                    .await?;
                self.users
                    .update_authentication_id_for_user(*id, authentication_id)
                    .await?;
            }

            state.sequence = message.sequence;
            current_job.update_execution_state(state.clone()).await?;
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
