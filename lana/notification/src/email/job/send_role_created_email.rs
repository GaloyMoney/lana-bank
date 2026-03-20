use async_trait::async_trait;
use domain_config::ExposedDomainConfigsReadOnly;
use serde::{Deserialize, Serialize};
use smtp_client::SmtpClient;

use job::*;
use tracing_macros::record_error_severity;

use crate::email::templates::{EmailTemplate, EmailType, RoleCreatedEmailData};

pub const SEND_ROLE_CREATED_EMAIL_COMMAND: JobType =
    JobType::new("command.notification.send-role-created-email");

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendRoleCreatedEmailConfig {
    pub role_id: core_access::RoleId,
    pub role_name: String,
    pub recipient_email: String,
}

pub struct SendRoleCreatedEmailInitializer {
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: ExposedDomainConfigsReadOnly,
}

impl SendRoleCreatedEmailInitializer {
    pub fn new(
        smtp_client: SmtpClient,
        template: EmailTemplate,
        domain_configs: ExposedDomainConfigsReadOnly,
    ) -> Self {
        Self {
            smtp_client,
            template,
            domain_configs,
        }
    }
}

impl JobInitializer for SendRoleCreatedEmailInitializer {
    type Config = SendRoleCreatedEmailConfig;

    fn job_type(&self) -> JobType {
        SEND_ROLE_CREATED_EMAIL_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(SendRoleCreatedEmailRunner {
            config: job.config()?,
            smtp_client: self.smtp_client.clone(),
            template: self.template.clone(),
            domain_configs: self.domain_configs.clone(),
        }))
    }
}

struct SendRoleCreatedEmailRunner {
    config: SendRoleCreatedEmailConfig,
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: ExposedDomainConfigsReadOnly,
}

#[async_trait]
impl JobRunner for SendRoleCreatedEmailRunner {
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

        super::send_rendered_email(
            &self.smtp_client,
            &self.template,
            &self.domain_configs,
            &self.config.recipient_email,
            &EmailType::RoleCreated(email_data),
        )
        .await?;

        Ok(JobCompletion::Complete)
    }
}
