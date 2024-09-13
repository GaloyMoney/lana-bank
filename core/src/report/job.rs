#![allow(clippy::blocks_in_conditions)]
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{job::*, primitives::ReportId};

use super::repo::ReportRepo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReportConfig {
    pub(super) report_id: ReportId,
}

pub struct GenerateReportInitializer {
    repo: ReportRepo,
}

impl GenerateReportInitializer {
    pub fn new(repo: &ReportRepo) -> Self {
        Self { repo: repo.clone() }
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
            repo: self.repo.clone(),
        }))
    }
}

pub struct GenerateReportJobRunner {
    config: GenerateReportConfig,
    repo: ReportRepo,
}

#[async_trait]
impl JobRunner for GenerateReportJobRunner {
    #[tracing::instrument(name = "lava.report.job.run", skip_all, fields(insert_id), err)]
    async fn run(&self, _: CurrentJob) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        unimplemented!()
    }
}
