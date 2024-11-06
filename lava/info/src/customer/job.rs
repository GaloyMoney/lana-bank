use async_trait::async_trait;
use chrono::Utc;
use futures::StreamExt;

use job::*;

use crate::Outbox;

use super::{repo::CustomerInfoRepo, values::*};

#[derive(serde::Serialize)]
pub struct CustomerInfoProjectionJobConfig;
impl JobConfig for CustomerInfoProjectionJobConfig {
    type Initializer = CustomerInfoProjectionJobInitializer;
}

pub struct CustomerInfoProjectionJobInitializer {
    outbox: Outbox,
    repo: CustomerInfoRepo,
}

impl CustomerInfoProjectionJobInitializer {
    pub fn new(outbox: &Outbox, repo: &CustomerInfoRepo) -> Self {
        Self {
            repo: repo.clone(),
            outbox: outbox.clone(),
        }
    }
}

const CUSTOMER_INFO_PROJECTION_JOB: JobType = JobType::new("customer-info-projection");
impl JobInitializer for CustomerInfoProjectionJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CUSTOMER_INFO_PROJECTION_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CustomerInfoProjectionJobRunner {
            outbox: self.outbox.clone(),
            repo: self.repo.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
struct CustomerInfoProjectionJobData {
    sequence: outbox::EventSequence,
    customer_info: CustomerInfoValues,
}

pub struct CustomerInfoProjectionJobRunner {
    outbox: Outbox,
    repo: CustomerInfoRepo,
}
#[async_trait]
impl JobRunner for CustomerInfoProjectionJobRunner {
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CustomerInfoProjectionJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            if let Some(payload) = &message.payload {
                if state
                    .customer_info
                    .process_event(message.recorded_at, payload)
                {
                    let mut db = self.repo.begin().await?;
                    self.repo
                        .persist_in_tx(&mut db, &state.customer_info)
                        .await?;
                    state.sequence = message.sequence;
                    current_job
                        .update_execution_state_in_tx(&mut db, &state)
                        .await?;
                    db.commit().await?;
                }
            }
        }

        Ok(JobCompletion::RescheduleAt(Utc::now()))
    }
}
