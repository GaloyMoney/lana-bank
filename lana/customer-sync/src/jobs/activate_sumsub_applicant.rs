use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use job::*;

use core_customer::CustomerId;
use tracing_macros::record_error_severity;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActivateSumsubApplicantConfig {
    pub customer_id: CustomerId,
}

pub const ACTIVATE_SUMSUB_APPLICANT_COMMAND: JobType =
    JobType::new("command.customer-sync.activate-sumsub-applicant");

pub struct ActivateSumsubApplicantJobInitializer {
    sumsub_client: sumsub::SumsubClient,
}

impl ActivateSumsubApplicantJobInitializer {
    pub fn new(sumsub_client: sumsub::SumsubClient) -> Self {
        Self { sumsub_client }
    }
}

impl JobInitializer for ActivateSumsubApplicantJobInitializer {
    type Config = ActivateSumsubApplicantConfig;

    fn job_type(&self) -> JobType {
        ACTIVATE_SUMSUB_APPLICANT_COMMAND
    }

    fn init(
        &self,
        job: &Job,
        _: JobSpawner<Self::Config>,
    ) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(ActivateSumsubApplicantJobRunner {
            config: job.config()?,
            sumsub_client: self.sumsub_client.clone(),
        }))
    }
}

pub struct ActivateSumsubApplicantJobRunner {
    config: ActivateSumsubApplicantConfig,
    sumsub_client: sumsub::SumsubClient,
}

#[async_trait]
impl JobRunner for ActivateSumsubApplicantJobRunner {
    #[record_error_severity]
    #[tracing::instrument(
        name = "customer_sync.activate_sumsub_applicant_job.process_command",
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
