#![allow(clippy::blocks_in_conditions)]
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{job::*, primitives::ReportId};

use super::{dataform_client::DataformClient, entity::Step, repo::ReportRepo, ReportConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReportConfig {
    pub(super) report_id: ReportId,
}

pub struct GenerateReportInitializer {
    repo: ReportRepo,
    report_config: ReportConfig,
}

impl GenerateReportInitializer {
    pub fn new(repo: &ReportRepo, report_config: &ReportConfig) -> Self {
        Self {
            repo: repo.clone(),
            report_config: report_config.clone(),
        }
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
            report_config: self.report_config.clone(),
        }))
    }
}

pub struct GenerateReportJobRunner {
    config: GenerateReportConfig,
    repo: ReportRepo,
    report_config: ReportConfig,
}

#[async_trait]
impl JobRunner for GenerateReportJobRunner {
    #[tracing::instrument(name = "lava.report.job.run", skip_all, fields(insert_id), err)]
    async fn run(&self, _: CurrentJob) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut report = self.repo.find_by_id(self.config.report_id).await?;
        // audit step needs to be added

        match report.next_step() {
            Step::Compilation => {
                let mut client = DataformClient::connect(&self.report_config).await?;
                match client.compile().await {
                    Ok(res) => {
                        report.compile(res);
                    }
                    Err(e) => {
                        report.compilation_failed(e.to_string());
                    }
                }
                self.repo.persist(&mut report).await?;
                return Ok(JobCompletion::RescheduleAt(chrono::Utc::now()));
            }

            Step::Invocation => {
                // Do invocation
            }

            Step::Upload => {
                // Do upload
            }
        }

        Ok(JobCompletion::Complete)
    }
}
