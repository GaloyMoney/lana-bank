use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use job::*;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditCollectionEvent, CoreCreditEvent, collateral::public::CoreCreditCollateralEvent,
    primitives::CreditFacilityId,
};

use super::super::repo::HistoryRepo;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum UpdateHistoryConfig {
    CreditEvent {
        facility_id: CreditFacilityId,
        recorded_at: DateTime<Utc>,
        event: serde_json::Value,
    },
    CollateralEvent {
        facility_id: CreditFacilityId,
        recorded_at: DateTime<Utc>,
        event: serde_json::Value,
    },
    CollectionEvent {
        facility_id: CreditFacilityId,
        event: serde_json::Value,
    },
}

impl UpdateHistoryConfig {
    pub(super) fn facility_id(&self) -> CreditFacilityId {
        match self {
            Self::CreditEvent { facility_id, .. }
            | Self::CollateralEvent { facility_id, .. }
            | Self::CollectionEvent { facility_id, .. } => *facility_id,
        }
    }
}

pub const UPDATE_HISTORY_COMMAND: JobType = JobType::new("command.credit.update-history");

pub struct UpdateHistoryJobInitializer {
    repo: Arc<HistoryRepo>,
}

impl UpdateHistoryJobInitializer {
    pub fn new(repo: Arc<HistoryRepo>) -> Self {
        Self { repo }
    }
}

impl JobInitializer for UpdateHistoryJobInitializer {
    type Config = UpdateHistoryConfig;

    fn job_type(&self) -> JobType {
        UPDATE_HISTORY_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdateHistoryJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
        }))
    }
}

struct UpdateHistoryJobRunner {
    config: UpdateHistoryConfig,
    repo: Arc<HistoryRepo>,
}

#[async_trait]
impl JobRunner for UpdateHistoryJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "credit.update_history_job.process_command",
        skip(self, current_job)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;

        let facility_id = self.config.facility_id();
        let mut history = self.repo.load(facility_id).await?;

        match &self.config {
            UpdateHistoryConfig::CreditEvent {
                recorded_at, event, ..
            } => {
                let credit_event: CoreCreditEvent = serde_json::from_value(event.clone())?;
                history.process_credit_event(&credit_event, *recorded_at);
            }
            UpdateHistoryConfig::CollateralEvent {
                recorded_at, event, ..
            } => {
                let collateral_event: CoreCreditCollateralEvent =
                    serde_json::from_value(event.clone())?;
                history.process_collateral_event(&collateral_event, *recorded_at);
            }
            UpdateHistoryConfig::CollectionEvent { event, .. } => {
                let collection_event: CoreCreditCollectionEvent =
                    serde_json::from_value(event.clone())?;
                history.process_collection_event(&collection_event);
            }
        }

        self.repo
            .persist_in_tx(op.tx_mut(), facility_id, history)
            .await?;
        Ok(JobCompletion::CompleteWithOp(op))
    }
}
