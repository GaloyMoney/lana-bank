use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;

use core_customer::CustomerId;
use tracing_macros::record_error_severity;

use super::sumsub_sync_job::complete_on_success;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RejectSumsubApplicantConfig {
    pub customer_id: CustomerId,
}

pub const REJECT_SUMSUB_APPLICANT_COMMAND: JobType =
    JobType::new("command.customer-sync.reject-sumsub-applicant");

pub struct RejectSumsubApplicantJobInitializer {
    sumsub_client: sumsub::SumsubClient,
}

impl RejectSumsubApplicantJobInitializer {
    pub fn new(sumsub_client: sumsub::SumsubClient) -> Self {
        Self { sumsub_client }
    }
}

impl JobInitializer for RejectSumsubApplicantJobInitializer {
    type Config = RejectSumsubApplicantConfig;

    fn job_type(&self) -> JobType {
        REJECT_SUMSUB_APPLICANT_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(RejectSumsubApplicantJobRunner {
            config: job.config()?,
            sumsub_client: self.sumsub_client.clone(),
        }))
    }
}

pub struct RejectSumsubApplicantJobRunner {
    config: RejectSumsubApplicantConfig,
    sumsub_client: sumsub::SumsubClient,
}

#[async_trait]
impl JobRunner for RejectSumsubApplicantJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "customer_sync.reject_sumsub_applicant_job.process_command",
        skip(self, _current_job),
        fields(customer_id = %self.config.customer_id),
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        complete_on_success(
            self.sumsub_client
                .reject_applicant(
                    self.config.customer_id,
                    "Customer account frozen by compliance",
                )
                .await,
        )
    }
}
