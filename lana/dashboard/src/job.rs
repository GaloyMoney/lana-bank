use async_trait::async_trait;
use futures::StreamExt;

use job::*;

use crate::{Outbox, repo::DashboardRepo, values::*};

#[derive(serde::Serialize)]
pub struct PermanentDashboardProjectionJobConfig;
impl JobConfig for PermanentDashboardProjectionJobConfig {
    type Initializer = PermanentDashboardProjectionInit;
}

pub struct PermanentDashboardProjectionInit {
    outbox: Outbox,
    repo: DashboardRepo,
}

impl PermanentDashboardProjectionInit {
    pub fn new(outbox: &Outbox, repo: &DashboardRepo) -> Self {
        Self {
            repo: repo.clone(),
            outbox: outbox.clone(),
        }
    }
}

const DASHBOARD_PROJECTION_JOB: JobType = JobType::new("permanent-dashboard-projection");
impl JobInitializer for PermanentDashboardProjectionInit {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        DASHBOARD_PROJECTION_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(PermanentDashboardProjectionJobRunner {
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
struct PermanentDashboardProjectionJobData {
    sequence: outbox::EventSequence,
    dashboard: DashboardValues,
}

pub struct PermanentDashboardProjectionJobRunner {
    outbox: Outbox,
    repo: DashboardRepo,
}
#[async_trait]
impl JobRunner for PermanentDashboardProjectionJobRunner {
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<PermanentDashboardProjectionJobData>()?
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
