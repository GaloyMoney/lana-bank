use async_trait::async_trait;
use domain_config::{DomainConfigError, DomainConfigs};
use job::*;
use serde::{Deserialize, Serialize};
use smtp_client::SmtpClient;
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::email::config::{NotificationFromEmailConfigSpec, NotificationFromNameConfigSpec};
use crate::email::templates::{EmailTemplate, EmailType};

#[derive(Serialize, Deserialize, Clone)]
pub struct EmailSenderConfig {
    pub recipient: String,
    pub email_type: EmailType,
}

pub struct EmailSenderInit {
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: DomainConfigs,
}

impl EmailSenderInit {
    pub fn new(
        smtp_client: SmtpClient,
        template: EmailTemplate,
        domain_configs: DomainConfigs,
    ) -> Self {
        Self {
            smtp_client,
            template,
            domain_configs,
        }
    }
}

const EMAIL_SENDER_JOB: JobType = JobType::new("task.email-sender");

impl JobInitializer for EmailSenderInit {
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

pub struct EmailSenderRunner {
    config: EmailSenderConfig,
    smtp_client: SmtpClient,
    template: EmailTemplate,
    domain_configs: DomainConfigs,
}

#[async_trait]
impl JobRunner for EmailSenderRunner {
    #[record_error_severity]
    #[instrument(name = "notification.email_sender_job.run", skip(self, _current_job), fields(recipient = %self.config.recipient, email_type = ?self.config.email_type))]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let from_email = match self
            .domain_configs
            .get::<NotificationFromEmailConfigSpec>()
            .await
        {
            Ok(email) => email,
            Err(DomainConfigError::NotConfigured) => {
                tracing::warn!("no configured notification from email; skipping email");
                return Ok(JobCompletion::Complete);
            }
            Err(e) => return Err(e.into()),
        };

        let from_name = match self
            .domain_configs
            .get::<NotificationFromNameConfigSpec>()
            .await
        {
            Ok(name) => name,
            Err(DomainConfigError::NotConfigured) => {
                tracing::warn!("no configured notification from name; skipping email");
                return Ok(JobCompletion::Complete);
            }
            Err(e) => return Err(e.into()),
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
