use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;

use core_customer::CustomerId;
use tracing_macros::record_error_severity;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApproveSumsubApplicantConfig {
    pub customer_id: CustomerId,
}

pub const APPROVE_SUMSUB_APPLICANT_COMMAND: JobType =
    JobType::new("command.customer-sync.approve-sumsub-applicant");

pub struct ApproveSumsubApplicantJobInitializer {
    sumsub_client: sumsub::SumsubClient,
}

impl ApproveSumsubApplicantJobInitializer {
    pub fn new(sumsub_client: sumsub::SumsubClient) -> Self {
        Self { sumsub_client }
    }
}

impl JobInitializer for ApproveSumsubApplicantJobInitializer {
    type Config = ApproveSumsubApplicantConfig;

    fn job_type(&self) -> JobType {
        APPROVE_SUMSUB_APPLICANT_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ApproveSumsubApplicantJobRunner {
            config: job.config()?,
            sumsub_client: self.sumsub_client.clone(),
        }))
    }
}

pub struct ApproveSumsubApplicantJobRunner {
    config: ApproveSumsubApplicantConfig,
    sumsub_client: sumsub::SumsubClient,
}

#[async_trait]
impl JobRunner for ApproveSumsubApplicantJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "customer_sync.approve_sumsub_applicant_job.process_command",
        skip(self, _current_job),
        fields(customer_id = %self.config.customer_id),
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        if let Err(e) = self
            .sumsub_client
            .approve_applicant(self.config.customer_id)
            .await
        {
            tracing::warn!("Failed to approve SumSub applicant: {e}");
        }

        Ok(JobCompletion::Complete)
    }
}
