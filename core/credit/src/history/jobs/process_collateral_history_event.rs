use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use job::*;
use tracing_macros::record_error_severity;

use crate::{collateral::public::CoreCreditCollateralEvent, primitives::CreditFacilityId};

use super::super::repo::HistoryRepo;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessCollateralHistoryEventConfig {
    pub facility_id: CreditFacilityId,
    pub recorded_at: DateTime<Utc>,
    pub event: CoreCreditCollateralEvent,
}

pub const PROCESS_COLLATERAL_HISTORY_EVENT_COMMAND: JobType =
    JobType::new("command.credit.process-collateral-history-event");

pub struct ProcessCollateralHistoryEventJobInitializer {
    repo: Arc<HistoryRepo>,
}

impl ProcessCollateralHistoryEventJobInitializer {
    pub fn new(repo: Arc<HistoryRepo>) -> Self {
        Self { repo }
    }
}

impl JobInitializer for ProcessCollateralHistoryEventJobInitializer {
    type Config = ProcessCollateralHistoryEventConfig;

    fn job_type(&self) -> JobType {
        PROCESS_COLLATERAL_HISTORY_EVENT_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ProcessCollateralHistoryEventJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
        }))
    }
}

struct ProcessCollateralHistoryEventJobRunner {
    config: ProcessCollateralHistoryEventConfig,
    repo: Arc<HistoryRepo>,
}

#[async_trait]
impl JobRunner for ProcessCollateralHistoryEventJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "credit.process_collateral_history_event_job.process_command",
        skip(self, current_job)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        let facility_id = self.config.facility_id;
        let mut history = self.repo.load(facility_id).await?;

        history.process_collateral_event(&self.config.event, self.config.recorded_at);

        self.repo
            .persist_in_tx(op.tx_mut(), facility_id, history)
            .await?;
        Ok(JobCompletion::CompleteWithOp(op))
    }
}
