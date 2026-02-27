use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use job::*;
use tracing_macros::record_error_severity;

use crate::{CoreCreditEvent, primitives::CreditFacilityId};

use super::super::repo::HistoryRepo;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessCreditHistoryEventConfig {
    pub facility_id: CreditFacilityId,
    pub recorded_at: DateTime<Utc>,
    pub event: CoreCreditEvent,
}

pub const PROCESS_CREDIT_HISTORY_EVENT_COMMAND: JobType =
    JobType::new("command.credit.process-credit-history-event");

pub struct ProcessCreditHistoryEventJobInitializer {
    repo: Arc<HistoryRepo>,
}

impl ProcessCreditHistoryEventJobInitializer {
    pub fn new(repo: Arc<HistoryRepo>) -> Self {
        Self { repo }
    }
}

impl JobInitializer for ProcessCreditHistoryEventJobInitializer {
    type Config = ProcessCreditHistoryEventConfig;

    fn job_type(&self) -> JobType {
        PROCESS_CREDIT_HISTORY_EVENT_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ProcessCreditHistoryEventJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
        }))
    }
}

struct ProcessCreditHistoryEventJobRunner {
    config: ProcessCreditHistoryEventConfig,
    repo: Arc<HistoryRepo>,
}

#[async_trait]
impl JobRunner for ProcessCreditHistoryEventJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "credit.process_credit_history_event_job.process_command",
        skip(self, current_job)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        let facility_id = self.config.facility_id;
        let mut history = self.repo.load(facility_id).await?;

        history.process_credit_event(&self.config.event, self.config.recorded_at);

        self.repo
            .persist_in_tx(op.tx_mut(), facility_id, history)
            .await?;
        Ok(JobCompletion::CompleteWithOp(op))
    }
}
