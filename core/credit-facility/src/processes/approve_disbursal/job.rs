use async_trait::async_trait;
use futures::StreamExt;

use governance::GovernanceEvent;
use job::*;
use lana_events::LanaEvent;

use super::ApproveDisbursal;
use crate::outbox::Outbox;

#[derive(serde::Serialize)]
pub(in crate::credit_facility) struct DisbursalApprovalJobConfig;
impl JobConfig for DisbursalApprovalJobConfig {
    type Initializer = DisbursalApprovalJobInitializer;
}

pub(in crate::credit_facility) struct DisbursalApprovalJobInitializer {
    outbox: Outbox,
    process: ApproveDisbursal,
}

impl DisbursalApprovalJobInitializer {
    pub fn new(outbox: &Outbox, process: &ApproveDisbursal) -> Self {
        Self {
            process: process.clone(),
            outbox: outbox.clone(),
        }
    }
}

const DISBURSAL_APPROVE_JOB: JobType = JobType::new("disbursal");
impl JobInitializer for DisbursalApprovalJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        DISBURSAL_APPROVE_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(DisbursalApprovalJobRunner {
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
struct DisbursalApprovalJobData {
    sequence: outbox::EventSequence,
}

pub struct DisbursalApprovalJobRunner {
    outbox: Outbox,
    process: ApproveDisbursal,
}
#[async_trait]
impl JobRunner for DisbursalApprovalJobRunner {
    #[allow(clippy::single_match)]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<DisbursalApprovalJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            match message.payload {
                Some(LanaEvent::Governance(GovernanceEvent::ApprovalProcessConcluded {
                    id,
                    approved,
                    ref process_type,
                    ..
                })) if process_type == &super::APPROVE_DISBURSAL_PROCESS => {
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
