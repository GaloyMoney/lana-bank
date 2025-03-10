use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use job::{Job, JobCompletion, JobConfig, JobId, JobInitializer, JobRunner, JobType, CurrentJob, Jobs, RetrySettings};
use lana_events::{LanaEvent, CoreCustomerEvent};
use outbox::Outbox;
use sqlx::PgPool;

use crate::email::sender_job::{EmailSenderJobConfig, EmailType};

#[derive(Serialize, Deserialize)]
pub struct EmailListenerJobConfig;
impl JobConfig for EmailListenerJobConfig {
    type Initializer = EmailListenerJobInitializer;
}

pub struct EmailListenerJobInitializer {
    pool: PgPool,
    outbox: Outbox<LanaEvent>,
    jobs: Jobs,
}

impl EmailListenerJobInitializer {
    pub fn new(pool: &PgPool, outbox: &Outbox<LanaEvent>, jobs: &Jobs) -> Self {
        Self {
            pool: pool.clone(),
            outbox: outbox.clone(),
            jobs: jobs.clone(),
        }
    }
}

const EMAIL_LISTENER_JOB: JobType = JobType::new("email-listener");
impl JobInitializer for EmailListenerJobInitializer {
    fn job_type() -> JobType {
        EMAIL_LISTENER_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EmailListenerJobRunner {
            pool: self.pool.clone(),
            outbox: self.outbox.clone(),
            jobs: self.jobs.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct EmailListenerJobData {
    sequence: outbox::EventSequence,
}

pub struct EmailListenerJobRunner {
    pool: PgPool,
    outbox: Outbox<LanaEvent>,
    jobs: Jobs,
}

#[async_trait]
impl JobRunner for EmailListenerJobRunner {
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<EmailListenerJobData>()?
            .unwrap_or_default();

        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;
        while let Some(message) = stream.next().await {
            if let Some(event) = &message.payload {
                if let Some(email_type) = self.map_event_to_email_type(event) {
                    let mut db = self.jobs.repo().begin_op().await?;
                    self.jobs
                        .create_and_spawn_in_op::<EmailSenderJobConfig>(
                            &mut db,
                            JobId::new(),
                            EmailSenderJobConfig { email_type },
                        )
                        .await?;
                    db.commit().await?;
                }
            }
            state.sequence = message.sequence;
            current_job.update_execution_state(state.clone()).await?;
        }
        Ok(JobCompletion::RescheduleNow)
    }
}

impl EmailListenerJobRunner {
    fn map_event_to_email_type(&self, event: &LanaEvent) -> Option<EmailType> {
        match event {
            LanaEvent::Customer(CoreCustomerEvent::CustomerCreated { customer_id, .. }) => {
                Some(EmailType::CustomerWelcome {
                    customer_id: *customer_id,
                })
            }
            _ => None,
        }
    }
}
