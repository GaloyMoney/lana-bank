use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;
use tracing_macros::record_error_severity;

use crate::{CoreCreditCollectionEvent, primitives::CreditFacilityId};

use super::super::repo::HistoryRepo;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCollectionHistoryConfig {
    pub facility_id: CreditFacilityId,
    pub event: CoreCreditCollectionEvent,
}

pub const UPDATE_COLLECTION_HISTORY_COMMAND: JobType =
    JobType::new("command.credit.update-collection-history");

pub struct UpdateCollectionHistoryJobInitializer {
    repo: Arc<HistoryRepo>,
}

impl UpdateCollectionHistoryJobInitializer {
    pub fn new(repo: Arc<HistoryRepo>) -> Self {
        Self { repo }
    }
}

impl JobInitializer for UpdateCollectionHistoryJobInitializer {
    type Config = UpdateCollectionHistoryConfig;

    fn job_type(&self) -> JobType {
        UPDATE_COLLECTION_HISTORY_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdateCollectionHistoryJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
        }))
    }
}

struct UpdateCollectionHistoryJobRunner {
    config: UpdateCollectionHistoryConfig,
    repo: Arc<HistoryRepo>,
}

#[async_trait]
impl JobRunner for UpdateCollectionHistoryJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "credit.update_collection_history_job.process_command",
        skip(self, current_job)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        let facility_id = self.config.facility_id;
        let mut history = self.repo.load(facility_id).await?;

        history.process_collection_event(&self.config.event);

        self.repo
            .persist_in_tx(op.tx_mut(), facility_id, history)
            .await?;
        Ok(JobCompletion::CompleteWithOp(op))
    }
}
