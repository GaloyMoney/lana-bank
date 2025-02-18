use async_trait::async_trait;
use futures::StreamExt;

use governance::GovernanceEvent;
use job::*;
use lana_events::LanaEvent;

use super::ApproveCreditFacility;
use crate::outbox::Outbox;

#[derive(serde::Serialize)]
pub(in crate::credit_facility) struct CreditFacilityApprovalJobConfig;
impl JobConfig for CreditFacilityApprovalJobConfig {
    type Initializer = CreditFacilityApprovalJobInitializer;
}

pub(in crate::credit_facility) struct CreditFacilityApprovalJobInitializer {
    outbox: Outbox,
    process: ApproveCreditFacility,
}

impl CreditFacilityApprovalJobInitializer {
    pub fn new(outbox: &Outbox, process: &ApproveCreditFacility) -> Self {
        Self {
            process: process.clone(),
            outbox: outbox.clone(),
        }
    }
}

const CREDIT_FACILITY_APPROVE_JOB: JobType = JobType::new("credit-facility");
impl JobInitializer for CreditFacilityApprovalJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREDIT_FACILITY_APPROVE_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreditFacilityApprovalJobRunner {
            outbox: self.outbox.clone(),
            process: self.process.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings
    where
        Self: Sized,
    {
        RetrySettings::repeat_indefinitely()
    }
}

#[derive(Default, Clone, Copy, serde::Deserialize, serde::Serialize)]
struct CreditFacilityApprovalJobData {
    sequence: outbox::EventSequence,
}

pub struct CreditFacilityApprovalJobRunner {
    outbox: Outbox,
    process: ApproveCreditFacility,
}
#[async_trait]
impl JobRunner for CreditFacilityApprovalJobRunner {
    #[allow(clippy::single_match)]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<CreditFacilityApprovalJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            match message.payload {
                Some(LanaEvent::Governance(GovernanceEvent::ApprovalProcessConcluded {
                    id,
                    approved,
                    ref process_type,
                    ..
                })) if process_type == &super::APPROVE_CREDIT_FACILITY_PROCESS => {
                    self.process.execute(id, approved).await?;
                    state.sequence = message.sequence;
                    current_job.update_execution_state(state).await?;
                }
                _ => {}
            }
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
