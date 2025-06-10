use async_trait::async_trait;
use job::{CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType};
use serde::{Deserialize, Serialize};

use crate::email::{smtp::SmtpClient, templates::EmailTemplate};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailSenderConfig {
    pub recipient: String,
    pub subject: String,
    pub template_data: String,
}

impl JobConfig for EmailSenderConfig {
    type Initializer = EmailSenderInitializer;
}

#[derive(Clone)]
pub struct EmailSenderInitializer {
    smtp_client: SmtpClient,
    template: EmailTemplate,
}

impl EmailSenderInitializer {
    pub fn new(smtp_client: SmtpClient, template: EmailTemplate) -> Self {
        Self {
            smtp_client,
            template,
        }
    }
}

const EMAIL_SENDER_JOB: JobType = JobType::new("email-sender");

impl JobInitializer for EmailSenderInitializer {
    fn job_type() -> JobType {
        EMAIL_SENDER_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EmailSenderRunner {
            config: job.config()?,
            smtp_client: self.smtp_client.clone(),
            template: self.template.clone(),
        }))
    }
}

pub struct EmailSenderRunner {
    config: EmailSenderConfig,
    smtp_client: SmtpClient,
    template: EmailTemplate,
}

#[async_trait]
impl JobRunner for EmailSenderRunner {
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let body = self
            .template
            .generic_email_template(&self.config.subject, &self.config.template_data)?;
        self.smtp_client
            .send_email(&self.config.recipient, &self.config.subject, body)
            .await?;
        Ok(JobCompletion::Complete)
    }
}
