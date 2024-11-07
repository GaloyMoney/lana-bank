use async_trait::async_trait;
use chrono::Utc;
use futures::StreamExt;

use job::*;

use crate::Outbox;

use super::{repo::CustomerSummaryRepo, values::*};

#[derive(serde::Serialize)]
pub struct CustomerSummaryProjectionJobConfig;
impl JobConfig for CustomerSummaryProjectionJobConfig {
    type Initializer = CustomerSummaryProjectionJobInitializer;
}

pub struct CustomerSummaryProjectionJobInitializer {
    outbox: Outbox,
    repo: CustomerSummaryRepo,
}

impl CustomerSummaryProjectionJobInitializer {
    pub fn new(outbox: &Outbox, repo: &CustomerSummaryRepo) -> Self {
        Self {
            repo: repo.clone(),
            outbox: outbox.clone(),
        }
    }
}

const CUSTOMER_SUMMARY_PROJECTION_JOB: JobType = JobType::new("customer-summary-projection");
impl JobInitializer for CustomerSummaryProjectionJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CUSTOMER_SUMMARY_PROJECTION_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CustomerSummaryProjectionJobRunner {
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
struct CustomerSummaryProjectionJobData {
    sequence: outbox::EventSequence,
    customer_summary: CustomerSummaryValues,
}

pub struct CustomerSummaryProjectionJobRunner {
    outbox: Outbox,
    repo: CustomerSummaryRepo,
}
#[async_trait]
impl JobRunner for CustomerSummaryProjectionJobRunner {
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CustomerSummaryProjectionJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            if let Some(payload) = &message.payload {
                if state
                    .customer_summary
                    .process_event(message.recorded_at, payload)
                {
                    let mut db = self.repo.begin().await?;
                    self.repo
                        .persist_in_tx(&mut db, &state.customer_summary)
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
