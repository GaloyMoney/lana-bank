use async_trait::async_trait;
use futures::StreamExt;

use job::*;

use crate::{Outbox, repo::DashboardRepo, values::*};

#[derive(serde::Serialize)]
pub struct DashboardProjectionJobConfig;
impl JobConfig for DashboardProjectionJobConfig {
    type Initializer = DashboardProjectionInit;
}

pub struct DashboardProjectionInit {
    outbox: Outbox,
    repo: DashboardRepo,
}

impl DashboardProjectionInit {
    pub fn new(outbox: &Outbox, repo: &DashboardRepo) -> Self {
        Self {
            repo: repo.clone(),
            outbox: outbox.clone(),
        }
    }
}

const DASHBOARD_PROJECTION_JOB: JobType = JobType::new("dashboard-projection");
impl JobInitializer for DashboardProjectionInit {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        DASHBOARD_PROJECTION_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(DashboardProjectionJobRunner {
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
struct DashboardProjectionJobData {
    sequence: outbox::EventSequence,
    dashboard: DashboardValues,
}

pub struct DashboardProjectionJobRunner {
    outbox: Outbox,
    repo: DashboardRepo,
}
#[async_trait]
impl JobRunner for DashboardProjectionJobRunner {
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<DashboardProjectionJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            if let Some(payload) = &message.payload
                && state.dashboard.process_event(message.recorded_at, payload)
            {
                let mut db = self.repo.begin().await?;
                self.repo.persist_in_tx(&mut db, &state.dashboard).await?;
                state.sequence = message.sequence;
                current_job
                    .update_execution_state_in_tx(&mut db, &state)
                    .await?;
                db.commit().await?;
            }
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
