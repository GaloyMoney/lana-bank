use async_trait::async_trait;
use job::{CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType};
use serde::{Deserialize, Serialize};

use crate::email::{executor::EmailExecutor, templates::EmailTemplate};

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
    executor: EmailExecutor,
    template: EmailTemplate,
}

impl EmailSenderInitializer {
    pub fn new(executor: EmailExecutor, template: EmailTemplate) -> Self {
        Self { executor, template }
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
            executor: self.executor.clone(),
            template: self.template.clone(),
        }))
    }
}

pub struct EmailSenderRunner {
    config: EmailSenderConfig,
    executor: EmailExecutor,
    template: EmailTemplate,
}

#[async_trait]
impl JobRunner for EmailSenderRunner {
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        self.executor
            .execute_email(
                &self.config.recipient,
                &self.config.subject,
                &self.config.template_data,
                &self.template,
            )
            .await?;
        Ok(JobCompletion::Complete)
    }
}
