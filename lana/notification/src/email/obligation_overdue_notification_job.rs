use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobId, JobInitializer, JobRunner, JobType, Jobs,
};

use crate::email::sender_job::EmailSenderJobConfig;

use audit::AuditSvc;
use audit::SystemSubject;

use core_access::event::CoreAccessEvent;
use core_access::user::Users;
use core_access::{CoreAccessAction, CoreAccessObject, UserId};
use outbox::OutboxEventMarker;

#[derive(Serialize, Deserialize)]
pub struct ObligationOverdueNotificationJobConfig<Audit, E> {
    pub obligation_id: Uuid,
    pub credit_facility_id: Uuid,
    #[serde(skip)]
    pub _phantom: std::marker::PhantomData<(Audit, E)>,
}

impl<Audit, E> ObligationOverdueNotificationJobConfig<Audit, E> {
    pub fn new(obligation_id: Uuid, credit_facility_id: Uuid) -> Self {
        Self {
            obligation_id,
            credit_facility_id,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Audit, E> JobConfig for ObligationOverdueNotificationJobConfig<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    type Initializer = ObligationOverdueNotificationJobInitializer<Audit, E>;
}

pub struct ObligationOverdueNotificationJobInitializer<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    users: Users<Audit, E>,
    jobs: Jobs,
}

impl<Audit, E> ObligationOverdueNotificationJobInitializer<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    pub fn new(users: &Users<Audit, E>, jobs: &Jobs) -> Self {
        Self {
            users: users.clone(),
            jobs: jobs.clone(),
        }
    }
}

const OBLIGATION_OVERDUE_NOTIFICATION_JOB: JobType =
    JobType::new("obligation-overdue-notification");

impl<Audit, E> JobInitializer for ObligationOverdueNotificationJobInitializer<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    fn job_type() -> JobType {
        OBLIGATION_OVERDUE_NOTIFICATION_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ObligationOverdueNotificationJobRunner {
            users: self.users.clone(),
            jobs: self.jobs.clone(),
            config: job.config()?,
        }))
    }
}

pub struct ObligationOverdueNotificationJobRunner<Audit, E>
where
    Audit: AuditSvc,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    users: Users<Audit, E>,
    jobs: Jobs,
    config: ObligationOverdueNotificationJobConfig<Audit, E>,
}

#[async_trait]
impl<Audit, E> JobRunner for ObligationOverdueNotificationJobRunner<Audit, E>
where
    Audit: AuditSvc,
    <Audit as AuditSvc>::Subject: From<UserId>,
    <Audit as AuditSvc>::Action: From<CoreAccessAction>,
    <Audit as AuditSvc>::Object: From<CoreAccessObject>,
    E: OutboxEventMarker<CoreAccessEvent>,
{
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        // list all users
        let subject = <Audit as AuditSvc>::Subject::system();
        let users = self.users.list_users(&subject).await?;
        for user in users {
            // email sender job for each user
            // TODO: build template for this email
            let email_config = EmailSenderJobConfig {
                recipient: user.email,
                subject: "Obligation overdue".to_string(),
                template_name: "obligation_overdue".to_string(),
                template_data: serde_json::json!({
                    "obligation_id": self.config.obligation_id,
                }),
            };
            let mut tx = self.jobs.begin_op().await?;
            self.jobs
                .create_and_spawn_in_op::<EmailSenderJobConfig>(&mut tx, JobId::new(), email_config)
                .await?;
            tx.commit().await?;
        }
        Ok(JobCompletion::Complete)
    }
}
