#![allow(clippy::blocks_in_conditions)]
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    audit::*,
    authorization::{Object, ReportAction},
    job::*,
    primitives::*,
    storage::Storage,
};

use crate::report::{ReportConfig, repo::ReportRepo, upload};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReportConfig {
    pub(in crate::report) report_id: ReportId,
}
impl JobConfig for GenerateReportConfig {
    type Initializer = GenerateReportInitializer;
}

pub struct GenerateReportInitializer {
    repo: ReportRepo,
    report_config: ReportConfig,
    audit: Audit,
    storage: Storage,
}

impl GenerateReportInitializer {
    pub fn new(
        repo: &ReportRepo,
        report_config: &ReportConfig,
        audit: &Audit,
        storage: &Storage,
    ) -> Self {
        Self {
            repo: repo.clone(),
            report_config: report_config.clone(),
            audit: audit.clone(),
            storage: storage.clone(),
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
            audit: self.audit.clone(),
            storage: self.storage.clone(),
        }))
    }
}

pub struct GenerateReportJobRunner {
    config: GenerateReportConfig,
    repo: ReportRepo,
    report_config: ReportConfig,
    audit: Audit,
    storage: Storage,
}

#[async_trait]
impl JobRunner for GenerateReportJobRunner {
    #[tracing::instrument(
        name = "lana.report.jobs.generate.run",
        skip_all,
        fields(insert_id),
        err
    )]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut report = self.repo.find_by_id(self.config.report_id).await?;

        let mut db = self.repo.begin_op().await?;

        let audit_info = self
            .audit
            .record_system_entry_in_tx(db.tx(), Object::all_reports(), ReportAction::Upload)
            .await?;

        match upload::execute(&self.report_config, &self.storage).await {
            Ok(files) => report.files_uploaded(files, audit_info),
            Err(e) => {
                report.upload_failed(e.to_string(), audit_info);

                self.repo.update_in_op(&mut db, &mut report).await?;
                db.commit().await?;

                return Ok(JobCompletion::RescheduleNow);
            }
        }

        self.repo.update_in_op(&mut db, &mut report).await?;
        db.commit().await?;

        Ok(JobCompletion::Complete)
    }
}
