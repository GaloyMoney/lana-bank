#![allow(clippy::blocks_in_conditions)]
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::job::*;

// use super::{cala::CalaClient, ExportData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReportConfig {
    // pub(super) cala_url: String,
    // pub(super) table_name: Cow<'static, str>,
    // pub(super) data: ExportData,
}

pub struct GenerateReportInitializer {}

impl GenerateReportInitializer {
    pub fn new() -> Self {
        Self {}
    }
}

const REPORT_JOB: JobType = JobType::new("generate-report");
impl JobInitializer for GenerateReportInitializer {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        REPORT_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(GenerateReportJobRunner {
            config: job.config()?,
        }))
    }
}

pub struct GenerateReportJobRunner {
    config: GenerateReportConfig,
}

#[async_trait]
impl JobRunner for GenerateReportJobRunner {
    #[tracing::instrument(name = "lava.report.job.run", skip_all, fields(insert_id), err)]
    async fn run(&self, _: CurrentJob) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        unimplemented!()
    }
}
