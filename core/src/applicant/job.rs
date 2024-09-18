use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use std::borrow::Cow;

use crate::{
    data_export::{Export, ExportSumsubApplicantData},
    job::*,
    primitives::CustomerId,
};

use super::SumsubClient;

#[derive(Clone, Deserialize, Serialize)]
pub struct SumsubExportConfig {
    pub customer_id: CustomerId,
}

pub struct SumsubExportInitializer {
    pub(super) export: Export,
    pub(super) sumsub_client: SumsubClient,
}

impl SumsubExportInitializer {
    pub fn new(export: Export, sumsub_client: SumsubClient) -> Self {
        Self {
            export,
            sumsub_client,
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
            config: job.config()?,
            export: self.export.clone(),
            sumsub_client: self.sumsub_client.clone(),
        }))
    }
}

pub struct SumsubExportJobRunner {
    config: SumsubExportConfig,
    export: Export,
    sumsub_client: SumsubClient,
}

#[async_trait]
impl JobRunner for SumsubExportJobRunner {
    #[tracing::instrument(name = "lava.sumsub_export.job.run", skip_all, fields(insert_id), err)]
    async fn run(&self, _: CurrentJob) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let customer_id = self.config.customer_id;

        let client = reqwest::Client::new();
        let res = self
            .sumsub_client
            .get_applicant_details(&client, customer_id)
            .await?;

        let _info = res.info;

        self.export
            .export_sum_sub_applicant_data(ExportSumsubApplicantData {
                customer_id,
                root: serde_json::to_string(&res).expect("Could not serialize res"),
                uploaded_at: Utc::now(),
            })
            .await?;

        Ok(JobCompletion::Complete)
    }
}
