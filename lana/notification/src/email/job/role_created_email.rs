use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use audit::AuditSvc;
use authz::PermissionCheck;
use core_access::user::Users;
use job::*;
use lana_events::LanaEvent;
use tracing_macros::record_error_severity;

use crate::email::job::sender::EmailSenderJobSpawner;
use crate::email::templates::{EmailType, RoleCreatedEmailData};

pub const ROLE_CREATED_EMAIL_COMMAND: JobType =
    JobType::new("command.notification.role-created-email");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoleCreatedEmailConfig {
    pub role_id: core_access::RoleId,
    pub role_name: String,
}

pub struct RoleCreatedEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    users: Users<Perms::Audit, LanaEvent>,
    email_sender_job_spawner: EmailSenderJobSpawner,
}

impl<Perms> RoleCreatedEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    pub fn new(
        users: &Users<Perms::Audit, LanaEvent>,
        email_sender_job_spawner: EmailSenderJobSpawner,
    ) -> Self {
        Self {
            users: users.clone(),
            email_sender_job_spawner,
        }
    }
}

impl<Perms> JobInitializer for RoleCreatedEmailInitializer<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<core_access::CoreAccessAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<core_access::CoreAccessObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: From<core_access::UserId>,
{
    type Config = RoleCreatedEmailConfig;

    fn job_type(&self) -> JobType {
        ROLE_CREATED_EMAIL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(RoleCreatedEmailRunner::<Perms> {
            config: job.config()?,
            users: self.users.clone(),
            email_sender_job_spawner: self.email_sender_job_spawner.clone(),
        }))
    }
}

struct RoleCreatedEmailRunner<Perms>
where
    Perms: PermissionCheck,
{
    config: RoleCreatedEmailConfig,
    users: Users<Perms::Audit, LanaEvent>,
    email_sender_job_spawner: EmailSenderJobSpawner,
}

#[async_trait]
impl<Perms> JobRunner for RoleCreatedEmailRunner<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<core_access::CoreAccessAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<core_access::CoreAccessObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: From<core_access::UserId>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "notification.role_created_email.run", skip_all)]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        let email_data = RoleCreatedEmailData {
            role_id: self.config.role_id.to_string(),
            role_name: self.config.role_name.clone(),
        };

        super::spawn_email_to_all_users(
            &self.users,
            &self.email_sender_job_spawner,
            &mut op,
            EmailType::RoleCreated(email_data),
        )
        .await?;

        Ok(JobCompletion::CompleteWithOp(op))
    }
}
