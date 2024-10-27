use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
    data_export::{Export, ExportSumsubApplicantData, SumsubContentType},
    job::*,
    primitives::CustomerId,
};

use super::{repo::ApplicantRepo, SumsubClient};

#[derive(Clone, Deserialize, Serialize)]
pub enum SumsubExportConfig {
    Webhook { callback_id: i64 },
    SensitiveInfo { customer_id: CustomerId },
}

pub struct SumsubExportInitializer {
    pub(super) export: Export,
    pub(super) sumsub_client: SumsubClient,
    pub(super) applicants: ApplicantRepo,
}

impl SumsubExportInitializer {
    pub fn new(export: Export, sumsub_client: SumsubClient, pool: &PgPool) -> Self {
        Self {
            export,
            sumsub_client,
            applicants: ApplicantRepo::new(pool),
        }
    }
}

const SUMSUB_EXPORT_JOB: JobType = JobType::new("sumsub-export");
impl JobInitializer for SumsubExportInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        SUMSUB_EXPORT_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(SumsubExportJobRunner {
            config: job.data()?,
            export: self.export.clone(),
            sumsub_client: self.sumsub_client.clone(),
            applicants: self.applicants.clone(),
        }))
    }
}

pub struct SumsubExportJobRunner {
    config: SumsubExportConfig,
    export: Export,
    sumsub_client: SumsubClient,
    applicants: ApplicantRepo,
}

#[async_trait]
impl JobRunner for SumsubExportJobRunner {
    #[tracing::instrument(name = "lava.sumsub_export.job.run", skip_all, fields(insert_id), err)]
    async fn run(&self, _: CurrentJob) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        match &self.config {
            SumsubExportConfig::Webhook { callback_id } => {
                let webhook_data = self
                    .applicants
                    .find_webhook_data_by_id(*callback_id)
                    .await?;

                self.export
                    .export_sum_sub_applicant_data(ExportSumsubApplicantData {
                        customer_id: webhook_data.customer_id,
                        content: serde_json::to_string(&webhook_data)?,
                        content_type: SumsubContentType::Webhook,
                        uploaded_at: webhook_data.timestamp,
                    })
                    .await?;

                Ok(JobCompletion::Complete)
            }
            SumsubExportConfig::SensitiveInfo { customer_id } => {
                let content = self
                    .sumsub_client
                    .get_applicant_details(*customer_id)
                    .await?;

                self.export
                    .export_sum_sub_applicant_data(ExportSumsubApplicantData {
                        customer_id: *customer_id,
                        content,
                        content_type: SumsubContentType::SensitiveInfo,
                        uploaded_at: chrono::Utc::now(),
                    })
                    .await?;

                Ok(JobCompletion::Complete)
            }
        }
    }
}
