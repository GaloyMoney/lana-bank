use async_trait::async_trait;
use domain_config::{DomainConfigValue, DomainConfigs};
use job::{CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType};
use serde::{Deserialize, Serialize};
use smtp_client::SmtpClient;
use tracing::instrument;
use tracing_macros::record_error_severity;

use crate::email::config::NotificationEmailConfig;
use crate::email::templates::{EmailTemplate, EmailType};

#[derive(Serialize, Deserialize)]
pub struct EmailSenderConfig {
    pub recipient: String,
    pub email_type: EmailType,
}

impl JobConfig for EmailSenderConfig {
    type Initializer = EmailSenderInit;
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
    fn job_type() -> JobType {
        EMAIL_SENDER_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
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
        let notification_email_conf = self
            .domain_configs
            .get_or_default::<NotificationEmailConfig>()
            .await?;

        if let Err(err) = notification_email_conf.validate() {
            tracing::warn!(error = ?err, "invalid notification email config; skipping email");
            return Ok(JobCompletion::Complete);
        }

        let (subject, body) = self.template.render_email(&self.config.email_type)?;
        self.smtp_client
            .send_email(
                &notification_email_conf.from_email,
                Some(&notification_email_conf.from_name),
                &self.config.recipient,
                &subject,
                body,
            )
            .await?;
        Ok(JobCompletion::Complete)
    }
}
