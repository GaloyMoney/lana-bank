use async_trait::async_trait;
use domain_config::ExposedDomainConfigsReadOnly;
use serde::{Deserialize, Serialize};
use smtp_client::SmtpClient;

use audit::AuditSvc;
use authz::PermissionCheck;
use core_access::user::Users;
use job::*;
use lana_events::LanaEvent;
use tracing_macros::record_error_severity;

use crate::email::templates::{EmailTemplate, EmailType, RoleCreatedEmailData};

pub const SEND_ROLE_CREATED_EMAIL_COMMAND: JobType =
    JobType::new("command.notification.send-role-created-email");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendRoleCreatedEmailConfig {
    pub role_id: core_access::RoleId,
    pub role_name: String,
}

pub struct SendRoleCreatedEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    users: Users<Perms::Audit, LanaEvent>,
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: ExposedDomainConfigsReadOnly,
}

impl<Perms> SendRoleCreatedEmailInitializer<Perms>
where
    Perms: PermissionCheck,
{
    pub fn new(
        users: &Users<Perms::Audit, LanaEvent>,
        smtp_client: SmtpClient,
        template: EmailTemplate,
        domain_configs: ExposedDomainConfigsReadOnly,
    ) -> Self {
        Self {
            users: users.clone(),
            smtp_client,
            template,
            domain_configs,
        }
    }
}

impl<Perms> JobInitializer for SendRoleCreatedEmailInitializer<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<core_access::CoreAccessAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<core_access::CoreAccessObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: From<core_access::UserId>,
{
    type Config = SendRoleCreatedEmailConfig;

    fn job_type(&self) -> JobType {
        SEND_ROLE_CREATED_EMAIL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(SendRoleCreatedEmailRunner::<Perms> {
            config: job.config()?,
            users: self.users.clone(),
            smtp_client: self.smtp_client.clone(),
            template: self.template.clone(),
            domain_configs: self.domain_configs.clone(),
        }))
    }
}

struct SendRoleCreatedEmailRunner<Perms>
where
    Perms: PermissionCheck,
{
    config: SendRoleCreatedEmailConfig,
    users: Users<Perms::Audit, LanaEvent>,
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: ExposedDomainConfigsReadOnly,
}

#[async_trait]
impl<Perms> JobRunner for SendRoleCreatedEmailRunner<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<core_access::CoreAccessAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<core_access::CoreAccessObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: From<core_access::UserId>,
{
    #[record_error_severity]
    #[tracing::instrument(name = "notification.send_role_created_email.run", skip_all)]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let email_data = RoleCreatedEmailData {
            role_id: self.config.role_id.to_string(),
            role_name: self.config.role_name.clone(),
        };

        super::send_email_to_all_users(
            &self.smtp_client,
            &self.template,
            &self.domain_configs,
            &self.users,
            &EmailType::RoleCreated(email_data),
        )
        .await?;

        Ok(JobCompletion::Complete)
    }
}
