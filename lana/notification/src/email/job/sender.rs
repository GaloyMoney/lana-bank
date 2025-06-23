use async_trait::async_trait;
use job::{CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType};
use serde::{Deserialize, Serialize};

use crate::email::{
    smtp::SmtpClient,
    templates::{EmailTemplate, EmailType},
};

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
}

impl EmailSenderInit {
    pub fn new(smtp_client: SmtpClient, template: EmailTemplate) -> Self {
        Self {
            smtp_client,
            template,
        }
    }
}

const EMAIL_SENDER_JOB: JobType = JobType::new("email-sender");

impl JobInitializer for EmailSenderInit {
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
        let (subject, body) = self.template.render_email(&self.config.email_type)?;
        self.smtp_client
            .send_email(&self.config.recipient, &subject, body)
            .await?;
        Ok(JobCompletion::Complete)
    }
}
