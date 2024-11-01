use async_trait::async_trait;
use futures::StreamExt;

use governance::GovernanceEvent;
use job::*;
use lava_events::LavaEvent;

use super::ApproveWithdraw;
use crate::outbox::Outbox;

#[derive(serde::Serialize)]
pub(in crate::withdraw) struct WithdrawApprovalJobConfig;
impl JobConfig for WithdrawApprovalJobConfig {
    type Initializer = WithdrawApprovalJobInitializer;
}

pub(in crate::withdraw) struct WithdrawApprovalJobInitializer {
    outbox: Outbox,
    process: ApproveWithdraw,
}

impl WithdrawApprovalJobInitializer {
    pub fn new(outbox: &Outbox, process: &ApproveWithdraw) -> Self {
        Self {
            process: process.clone(),
            outbox: outbox.clone(),
        }
    }
}

const WITHDRAW_APPROVE_JOB: JobType = JobType::new("withdraw-approval");
impl JobInitializer for WithdrawApprovalJobInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        WITHDRAW_APPROVE_JOB
    }

    fn init(&self, _: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(WithdrawApprovalJobRunner {
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
struct WithdrawApprovalJobData {
    sequence: outbox::EventSequence,
}

pub struct WithdrawApprovalJobRunner {
    outbox: Outbox,
    process: ApproveWithdraw,
}
#[async_trait]
impl JobRunner for WithdrawApprovalJobRunner {
    #[allow(clippy::single_match)]
    async fn run(
        &self,
        mut current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut state = current_job
            .execution_state::<WithdrawApprovalJobData>()?
            .unwrap_or_default();
        let mut stream = self.outbox.listen_persisted(Some(state.sequence)).await?;

        while let Some(message) = stream.next().await {
            match message.payload {
                Some(LavaEvent::Governance(GovernanceEvent::ApprovalProcessConcluded {
                    id,
                    approved,
                    ref process_type,
                    ..
                })) if process_type == &super::APPROVE_WITHDRAW_PROCESS => {
                    self.process.execute_from_job(id, approved).await?;
                    state.sequence = message.sequence;
                    current_job.update_execution_state(state).await?;
                }
                _ => {}
            }
        }

        Ok(JobCompletion::RescheduleAt(chrono::Utc::now()))
    }
}
