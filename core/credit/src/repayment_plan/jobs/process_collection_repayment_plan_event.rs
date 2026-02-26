use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use es_entity::AtomicOperation;
use job::*;
use obix::EventSequence;
use tracing_macros::record_error_severity;

use crate::{
    CoreCreditCollectionEvent, primitives::CreditFacilityId, repayment_plan::RepaymentPlanRepo,
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessCollectionRepaymentPlanEventConfig {
    pub facility_id: CreditFacilityId,
    pub sequence: EventSequence,
    pub event: CoreCreditCollectionEvent,
    pub trace_context: tracing_utils::persistence::SerializableTraceContext,
}

pub const PROCESS_COLLECTION_REPAYMENT_PLAN_EVENT_COMMAND: JobType =
    JobType::new("command.credit.process-collection-repayment-plan-event");

pub struct ProcessCollectionRepaymentPlanEventJobInitializer {
    repo: Arc<RepaymentPlanRepo>,
}

impl ProcessCollectionRepaymentPlanEventJobInitializer {
    pub fn new(repo: Arc<RepaymentPlanRepo>) -> Self {
        Self { repo }
    }
}

impl JobInitializer for ProcessCollectionRepaymentPlanEventJobInitializer {
    type Config = ProcessCollectionRepaymentPlanEventConfig;

    fn job_type(&self) -> JobType {
        PROCESS_COLLECTION_REPAYMENT_PLAN_EVENT_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ProcessCollectionRepaymentPlanEventJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
        }))
    }
}

struct ProcessCollectionRepaymentPlanEventJobRunner {
    config: ProcessCollectionRepaymentPlanEventConfig,
    repo: Arc<RepaymentPlanRepo>,
}

#[async_trait]
impl JobRunner for ProcessCollectionRepaymentPlanEventJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "credit.process_collection_repayment_plan_event_job.process_command",
        skip(self, current_job)
    )]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        tracing_utils::persistence::set_parent(&self.config.trace_context);
        let mut op = current_job.begin_op().await?;
        let clock = op.clock().clone();
        let now = clock.now();

        let facility_id = self.config.facility_id;
        let mut repayment_plan = self.repo.load(facility_id).await?;

        repayment_plan.process_collection_event(self.config.sequence, &self.config.event, now);

        self.repo
            .persist_in_tx(op.tx_mut(), facility_id, repayment_plan)
            .await?;
        Ok(JobCompletion::CompleteWithOp(op))
    }
}
