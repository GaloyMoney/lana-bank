use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ::job::*;

use crate::email::EmailNotification;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EmailType {
    CustomerWelcome { customer_id: Uuid },
}

#[derive(Serialize, Deserialize)]
pub struct EmailSenderJobConfig {
    pub email_type: EmailType,
}

impl JobConfig for EmailSenderJobConfig {
    type Initializer = EmailSenderJobInitializer;
}

pub struct EmailSenderJobInitializer {
    notification: EmailNotification,
}

impl EmailSenderJobInitializer {
    pub fn new(notification: EmailNotification) -> Self {
        Self { notification }
    }
}

const EMAIL_SENDER_JOB: JobType = JobType::new("email-sender");
impl JobInitializer for EmailSenderJobInitializer {
    fn job_type() -> JobType {
        EMAIL_SENDER_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let config = job.config::<EmailSenderJobConfig>()?;

        Ok(Box::new(EmailSenderJobRunner {
            notification: self.notification.clone(),
            config,
        }))
    }
}

pub struct EmailSenderJobRunner {
    notification: EmailNotification,
    config: EmailSenderJobConfig,
}

#[async_trait]
impl JobRunner for EmailSenderJobRunner {
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        match &self.config.email_type {
            EmailType::CustomerWelcome { customer_id } => {
                self.notification.send_customer_welcome(customer_id).await?
            }
        }
        Ok(JobCompletion::Complete)
    }
}
