use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use es_entity::AtomicOperation;
use job::*;
use obix::EventSequence;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditCollectionEvent, CoreCreditEvent, primitives::CreditFacilityId,
    repayment_plan::RepaymentPlanRepo,
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum UpdateRepaymentPlanConfig {
    Credit {
        facility_id: CreditFacilityId,
        sequence: EventSequence,
        recorded_at: DateTime<Utc>,
        event: serde_json::Value,
    },
    Collection {
        facility_id: CreditFacilityId,
        sequence: EventSequence,
        event: serde_json::Value,
    },
}

impl UpdateRepaymentPlanConfig {
    pub(super) fn facility_id(&self) -> CreditFacilityId {
        match self {
            Self::Credit { facility_id, .. } | Self::Collection { facility_id, .. } => *facility_id,
        }
    }
}

pub const UPDATE_REPAYMENT_PLAN_COMMAND: JobType =
    JobType::new("command.credit.update-repayment-plan");

pub struct UpdateRepaymentPlanJobInitializer {
    repo: Arc<RepaymentPlanRepo>,
}

impl UpdateRepaymentPlanJobInitializer {
    pub fn new(repo: Arc<RepaymentPlanRepo>) -> Self {
        Self { repo }
    }
}

impl JobInitializer for UpdateRepaymentPlanJobInitializer {
    type Config = UpdateRepaymentPlanConfig;

    fn job_type(&self) -> JobType {
        UPDATE_REPAYMENT_PLAN_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(UpdateRepaymentPlanJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
        }))
    }
}

struct UpdateRepaymentPlanJobRunner {
    config: UpdateRepaymentPlanConfig,
    repo: Arc<RepaymentPlanRepo>,
}

#[async_trait]
impl JobRunner for UpdateRepaymentPlanJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "credit.update_repayment_plan_job.process_command",
        skip(self, current_job)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut op = current_job.begin_op().await?;
        let clock = op.clock().clone();
        let now = clock.now();

        let facility_id = self.config.facility_id();
        let mut repayment_plan = self.repo.load(facility_id).await?;

        match &self.config {
            UpdateRepaymentPlanConfig::Credit {
                sequence,
                recorded_at,
                event,
                ..
            } => {
                let credit_event: CoreCreditEvent = serde_json::from_value(event.clone())?;
                repayment_plan.process_credit_event(*sequence, &credit_event, now, *recorded_at);
            }
            UpdateRepaymentPlanConfig::Collection {
                sequence, event, ..
            } => {
                let collection_event: CoreCreditCollectionEvent =
                    serde_json::from_value(event.clone())?;
                repayment_plan.process_collection_event(*sequence, &collection_event, now);
            }
        }

        self.repo
            .persist_in_tx(op.tx_mut(), facility_id, repayment_plan)
            .await?;
        Ok(JobCompletion::CompleteWithOp(op))
    }
}
