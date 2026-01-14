use async_trait::async_trait;
use audit::{AuditSvc, SystemSubject};
use authz::PermissionCheck;
use domain_config::ExposedDomainConfigs;
use job::*;
use serde::{Deserialize, Serialize};
use smtp_client::SmtpClient;
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::email::config::{NotificationFromEmail, NotificationFromName};
use crate::email::templates::{EmailTemplate, EmailType};

#[derive(Serialize, Deserialize, Clone)]
pub struct EmailSenderConfig {
    pub recipient: String,
    pub email_type: EmailType,
}

pub struct EmailSenderInit<Perms>
where
    Perms: PermissionCheck,
{
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: ExposedDomainConfigs<Perms>,
}

impl<Perms> EmailSenderInit<Perms>
where
    Perms: PermissionCheck,
{
    pub fn new(
        smtp_client: SmtpClient,
        template: EmailTemplate,
        domain_configs: ExposedDomainConfigs<Perms>,
    ) -> Self {
        Self {
            smtp_client,
            template,
            domain_configs,
        }
    }
}

const EMAIL_SENDER_JOB: JobType = JobType::new("task.email-sender");

impl<Perms> JobInitializer for EmailSenderInit<Perms>
where
    Perms: PermissionCheck + Clone + Send + Sync + 'static,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<domain_config::DomainConfigAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<domain_config::DomainConfigObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: SystemSubject,
{
    type Config = EmailSenderConfig;
    fn job_type(&self) -> JobType {
        EMAIL_SENDER_JOB
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EmailSenderRunner {
            config: job.config()?,
            smtp_client: self.smtp_client.clone(),
            template: self.template.clone(),
            domain_configs: self.domain_configs.clone(),
        }))
    }
}

pub struct EmailSenderRunner<Perms>
where
    Perms: PermissionCheck,
{
    config: EmailSenderConfig,
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: ExposedDomainConfigs<Perms>,
}

#[async_trait]
impl<Perms> JobRunner for EmailSenderRunner<Perms>
where
    Perms: PermissionCheck + Clone + Send + Sync + 'static,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action:
        From<domain_config::DomainConfigAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object:
        From<domain_config::DomainConfigObject>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject: SystemSubject,
{
    #[record_error_severity]
    #[instrument(name = "notification.email_sender_job.run", skip(self, _current_job), fields(recipient = %self.config.recipient, email_type = ?self.config.email_type))]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let system_sub = <<Perms as PermissionCheck>::Audit as AuditSvc>::Subject::system();
        let from_email_config = self
            .domain_configs
            .get::<NotificationFromEmail>(&system_sub)
            .await?;
        let from_email = match from_email_config.value() {
            Some(from_email) => from_email,
            None => {
                tracing::warn!("no configured notification from email; skipping email");
                return Ok(JobCompletion::Complete);
            }
        };

        let from_name_config = self
            .domain_configs
            .get::<NotificationFromName>(&system_sub)
            .await?;
        let from_name = match from_name_config.value() {
            Some(from_name) => from_name,
            None => {
                tracing::warn!("no configured notification from name; skipping email");
                return Ok(JobCompletion::Complete);
            }
        };

        let (subject, body) = self.template.render_email(&self.config.email_type)?;
        self.smtp_client
            .send_email(
                &from_email,
                Some(&from_name),
                &self.config.recipient,
                &subject,
                body,
            )
            .await?;
        Ok(JobCompletion::Complete)
    }
}

pub type EmailSenderJobSpawner = JobSpawner<EmailSenderConfig>;
