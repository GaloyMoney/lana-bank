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
pub struct UpdateCreditHistoryConfig {
    pub facility_id: CreditFacilityId,
    pub recorded_at: DateTime<Utc>,
    pub event: CoreCreditEvent,
}

pub const UPDATE_CREDIT_HISTORY_COMMAND: JobType =
    JobType::new("command.credit.update-credit-history");

pub struct UpdateCreditHistoryJobInitializer {
    repo: Arc<HistoryRepo>,
}

impl UpdateCreditHistoryJobInitializer {
    pub fn new(repo: Arc<HistoryRepo>) -> Self {
        Self { repo }
    }
}

impl JobInitializer for UpdateCreditHistoryJobInitializer {
    type Config = UpdateCreditHistoryConfig;

    fn job_type(&self) -> JobType {
        UPDATE_CREDIT_HISTORY_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdateCreditHistoryJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
        }))
    }
}

struct UpdateCreditHistoryJobRunner {
    config: UpdateCreditHistoryConfig,
    repo: Arc<HistoryRepo>,
}

#[async_trait]
impl JobRunner for UpdateCreditHistoryJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "credit.update_credit_history_job.process_command",
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
