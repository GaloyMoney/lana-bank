use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use ::job::*;
use outbox::Outbox;

use crate::email::{event::EmailEvent, executor::EmailExecutor, templates::EmailTemplate};

#[derive(Serialize, Deserialize)]
pub struct EmailJobConfig;
impl JobConfig for EmailJobConfig {
    type Initializer = EmailJobInitializer;
}

pub struct EmailJobInitializer {
    pool: PgPool,
    outbox: Outbox<EmailEvent>,
    executor: EmailExecutor,
    templates_path: String,
}

impl EmailJobInitializer {
    pub fn new(pool: &PgPool, outbox: &Outbox<EmailEvent>, executor: EmailExecutor) -> Self {
        Self {
            pool: pool.clone(),
            outbox: outbox.clone(),
            executor,
            templates_path: "./templates/email".to_string(),
        }
    }
}

pub const EMAIL_JOB: JobType = JobType::new("email-send");
impl JobInitializer for EmailJobInitializer {
    fn job_type() -> JobType {
        EMAIL_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(EmailJobRunner {
            pool: self.pool.clone(),
            executor: self.executor.clone(),
            outbox: self.outbox.clone(),
            templates_path: self.templates_path.clone(),
        }))
    }
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
struct EmailJobState {
    sequence: outbox::EventSequence,
}
pub struct EmailJobRunner {
    pool: PgPool,
    executor: EmailExecutor,
    outbox: Outbox<EmailEvent>,
    templates_path: String,
}

#[async_trait]
impl JobRunner for EmailJobRunner {
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<EmailJobState>()?
            .unwrap_or_default();

        let template = EmailTemplate::new(&self.templates_path)?;
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(persistent_event) = stream.next().await {
            if let Some(payload) = &persistent_event.payload {
                match payload {
                    EmailEvent::Requested {
                        id,
                        recipient,
                        subject,
                        template_name,
                        template_data,
                        ..
                    } => {
                        match self
                            .executor
                            .execute_email(
                                recipient,
                                subject,
                                template_name,
                                template_data,
                                &template,
                            )
                            .await
                        {
                            Ok(_) => {
                                let mut tx = self.pool.begin().await?;
                                self.outbox
                                    .publish_persisted(
                                        &mut tx,
                                        EmailEvent::Sent {
                                            id: *id,
                                            timestamp: chrono::Utc::now(),
                                        },
                                    )
                                    .await?;
                                tx.commit().await?;
                            }
                            Err(e) => {
                                let attempt = current_job.attempt();
                                let mut tx = self.pool.begin().await?;
                                self.outbox
                                    .publish_persisted(
                                        &mut tx,
                                        EmailEvent::Failed {
                                            id: *id,
                                            error: e.to_string(),
                                            attempt,
                                            timestamp: chrono::Utc::now(),
                                        },
                                    )
                                    .await?;
                                tx.commit().await?;
                                return Err(e.into());
                            }
                        }
                    }
                    _ => {}
                }
            }
            state.sequence = persistent_event.sequence;
            current_job.update_execution_state(state.clone()).await?;
        }
        Ok(JobCompletion::RescheduleIn(std::time::Duration::from_secs(
            5,
        )))
    }
}
