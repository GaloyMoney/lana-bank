use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::email::{executor::EmailExecutor, templates::EmailTemplate};
use ::job::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailSenderJobConfig {
    pub recipient: String,
    pub subject: String,
    pub template_name: String,
    pub template_data: serde_json::Value,
}

impl JobConfig for EmailSenderJobConfig {
    type Initializer = EmailSenderJobInitializer;
}

#[derive(Clone)]
pub struct EmailSenderJobInitializer {
    executor: EmailExecutor,
    template: EmailTemplate,
}

impl EmailSenderJobInitializer {
    pub fn new(executor: EmailExecutor, template: EmailTemplate) -> Self {
        Self { executor, template }
    }
}

const EMAIL_SENDER_JOB: JobType = JobType::new("email-sender");

impl JobInitializer for EmailSenderJobInitializer {
    fn job_type() -> JobType {
        EMAIL_SENDER_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EmailSenderJobRunner {
            config: job.config()?,
            executor: self.executor.clone(),
            template: self.template.clone(),
        }))
    }
}

pub struct EmailSenderJobRunner {
    config: EmailSenderJobConfig,
    executor: EmailExecutor,
    template: EmailTemplate,
}

#[async_trait]
impl JobRunner for EmailSenderJobRunner {
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.executor
            .execute_email(
                &self.config.recipient,
                &self.config.subject,
                &self.config.template_name,
                &self.config.template_data,
                &self.template,
            )
            .await?;
        Ok(JobCompletion::Complete)
    }
}
