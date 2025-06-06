use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
pub struct EmailSenderJobInitializer;

impl EmailSenderJobInitializer {
    pub fn new() -> Self {
        Self
    }
}

const EMAIL_SENDER_JOB: JobType = JobType::new("email-sender");

impl JobInitializer for EmailSenderJobInitializer {
    fn job_type() -> JobType {
        EMAIL_SENDER_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let config = job.config::<EmailSenderJobConfig>()?;

        Ok(Box::new(EmailSenderJobRunner { config }))
    }
}

pub struct EmailSenderJobRunner {
    config: EmailSenderJobConfig,
}

#[async_trait]
impl JobRunner for EmailSenderJobRunner {
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        println!(
            "Sending email to {}: subject={}, template={}, data={:?}",
            self.config.recipient,
            self.config.subject,
            self.config.template_name,
            self.config.template_data
        );
        Ok(JobCompletion::Complete)
    }
}
