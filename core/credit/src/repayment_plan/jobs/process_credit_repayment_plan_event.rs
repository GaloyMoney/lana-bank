use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use es_entity::AtomicOperation;
use job::*;
use obix::EventSequence;
use tracing_macros::record_error_severity;

use crate::{CoreCreditEvent, primitives::CreditFacilityId, repayment_plan::RepaymentPlanRepo};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessCreditRepaymentPlanEventConfig {
    pub facility_id: CreditFacilityId,
    pub sequence: EventSequence,
    pub recorded_at: DateTime<Utc>,
    pub event: CoreCreditEvent,
    pub trace_context: tracing_utils::persistence::SerializableTraceContext,
}

pub const PROCESS_CREDIT_REPAYMENT_PLAN_EVENT_COMMAND: JobType =
    JobType::new("command.credit.process-credit-repayment-plan-event");

pub struct ProcessCreditRepaymentPlanEventJobInitializer {
    repo: Arc<RepaymentPlanRepo>,
}

impl ProcessCreditRepaymentPlanEventJobInitializer {
    pub fn new(repo: Arc<RepaymentPlanRepo>) -> Self {
        Self { repo }
    }
}

impl JobInitializer for ProcessCreditRepaymentPlanEventJobInitializer {
    type Config = ProcessCreditRepaymentPlanEventConfig;

    fn job_type(&self) -> JobType {
        PROCESS_CREDIT_REPAYMENT_PLAN_EVENT_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ProcessCreditRepaymentPlanEventJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
        }))
    }
}

struct ProcessCreditRepaymentPlanEventJobRunner {
    config: ProcessCreditRepaymentPlanEventConfig,
    repo: Arc<RepaymentPlanRepo>,
}

#[async_trait]
impl JobRunner for ProcessCreditRepaymentPlanEventJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "credit.process_credit_repayment_plan_event_job.process_command",
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

        repayment_plan.process_credit_event(
            self.config.sequence,
            &self.config.event,
            now,
            self.config.recorded_at,
        );

        self.repo
            .persist_in_tx(op.tx_mut(), facility_id, repayment_plan)
            .await?;
        Ok(JobCompletion::CompleteWithOp(op))
    }
}
