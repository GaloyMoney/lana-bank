use async_trait::async_trait;
use futures::StreamExt;
use tokio::select;
use tracing::{Span, instrument};

use job::*;
use obix::out::PersistentOutboxEvent;

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

const DASHBOARD_PROJECTION_JOB: JobType = JobType::new("outbox.dashboard-projection");
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
    sequence: obix::EventSequence,
    dashboard: DashboardValues,
}

pub struct DashboardProjectionJobRunner {
    outbox: Outbox,
    repo: DashboardRepo,
}

impl DashboardProjectionJobRunner {
    #[instrument(name = "dashboard.projection_job.process_message", parent = None, skip(self, message, dashboard), fields(seq = %message.sequence, handled = false))]
    async fn process_message(
        &self,
        message: &PersistentOutboxEvent<lana_events::LanaEvent>,
        dashboard: &mut DashboardValues,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(payload) = &message.payload
            && dashboard.process_event(message.recorded_at, payload)
        {
            message.inject_trace_parent();
            Span::current().record("handled", true);
        }
        Ok(())
    }
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
        let mut stream = self.outbox.listen_persisted(Some(state.sequence));

        loop {
            select! {
                biased;

                _ = current_job.shutdown_requested() => {
                    tracing::info!(
                        job_id = %current_job.id(),
                        job_type = %DASHBOARD_PROJECTION_JOB,
                        last_sequence = %state.sequence,
                        "Shutdown signal received"
                    );
                    return Ok(JobCompletion::RescheduleNow);
                }
                message = stream.next() => {
                    match message {
                        Some(message) => {
                            let mut db = self.repo.begin().await?;
                            self.process_message(&message, &mut state.dashboard).await?;
                            self.repo.persist_in_tx(&mut db, &state.dashboard).await?;

                            state.sequence = message.sequence;
                            current_job
                                .update_execution_state_in_op(&mut db, &state)
                                .await?;

                            db.commit().await?;
                        }
                        None => {
                            return Ok(JobCompletion::RescheduleNow);
                        }
                    }
                }
            }
        }
    }
}
