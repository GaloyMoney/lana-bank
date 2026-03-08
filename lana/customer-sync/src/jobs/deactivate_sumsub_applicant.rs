use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;

use core_customer::CustomerId;
use tracing_macros::record_error_severity;

use super::sumsub_sync_job::complete_on_success;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeactivateSumsubApplicantConfig {
    pub customer_id: CustomerId,
}

pub const DEACTIVATE_SUMSUB_APPLICANT_COMMAND: JobType =
    JobType::new("command.customer-sync.deactivate-sumsub-applicant");

pub struct DeactivateSumsubApplicantJobInitializer {
    sumsub_client: sumsub::SumsubClient,
}

impl DeactivateSumsubApplicantJobInitializer {
    pub fn new(sumsub_client: sumsub::SumsubClient) -> Self {
        Self { sumsub_client }
    }
}

impl JobInitializer for DeactivateSumsubApplicantJobInitializer {
    type Config = DeactivateSumsubApplicantConfig;

    fn job_type(&self) -> JobType {
        DEACTIVATE_SUMSUB_APPLICANT_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(DeactivateSumsubApplicantJobRunner {
            config: job.config()?,
            sumsub_client: self.sumsub_client.clone(),
        }))
    }
}

pub struct DeactivateSumsubApplicantJobRunner {
    config: DeactivateSumsubApplicantConfig,
    sumsub_client: sumsub::SumsubClient,
}

#[async_trait]
impl JobRunner for DeactivateSumsubApplicantJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "customer_sync.deactivate_sumsub_applicant_job.process_command",
        skip(self, _current_job),
        fields(customer_id = %self.config.customer_id),
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        complete_on_success(
            self.sumsub_client
                .deactivate_applicant(self.config.customer_id)
                .await,
        )
    }
}
