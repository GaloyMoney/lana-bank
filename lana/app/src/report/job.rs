#![allow(clippy::blocks_in_conditions)]

use async_trait::async_trait;
use chrono::{DateTime, Datelike, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    audit::*,
    authorization::{Object, ReportAction},
    job::*,
    primitives::*,
    storage::Storage,
};

use crate::report::{NewReport, ReportConfig, repo::ReportRepo, upload};

// Generate Report functionality

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReportConfig {
    pub(in crate::report) report_id: ReportId,
}
impl JobConfig for GenerateReportConfig {
    type Initializer = GenerateReportInit;
}

pub struct GenerateReportInit {
    repo: ReportRepo,
    report_config: ReportConfig,
    audit: Audit,
    storage: Storage,
}

impl GenerateReportInit {
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
impl JobInitializer for GenerateReportInit {
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

// Create Report functionality

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateReportJobConfig {
    pub job_interval: CreateReportInterval,
}
impl JobConfig for CreateReportJobConfig {
    type Initializer = CreateReportInit;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CreateReportInterval {
    EndOfDay,
}

impl CreateReportInterval {
    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            CreateReportInterval::EndOfDay => {
                let d = Utc::now();
                Utc.with_ymd_and_hms(d.year(), d.month(), d.day(), 23, 59, 59)
                    .single()
                    .expect("should return a valid date time")
            }
        }
    }
}

pub struct CreateReportInit {
    repo: ReportRepo,
    jobs: Jobs,
    audit: Audit,
}

impl CreateReportInit {
    pub fn new(repo: &ReportRepo, jobs: &Jobs, audit: &Audit) -> Self {
        Self {
            repo: repo.clone(),
            jobs: jobs.clone(),
            audit: audit.clone(),
        }
    }
}

const CREATE_REPORT_JOB: JobType = JobType::new("create-report");
impl JobInitializer for CreateReportInit {
    fn job_type() -> JobType
    where
        Self: Sized,
    {
        CREATE_REPORT_JOB
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        Ok(Box::new(CreateReportJobRunner {
            config: job.config()?,
            repo: self.repo.clone(),
            jobs: self.jobs.clone(),
            audit: self.audit.clone(),
        }))
    }
}

pub struct CreateReportJobRunner {
    config: CreateReportJobConfig,
    repo: ReportRepo,
    jobs: Jobs,
    audit: Audit,
}

#[async_trait]
impl JobRunner for CreateReportJobRunner {
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut db = self.repo.begin_op().await?;

        let audit_info = self
            .audit
            .record_system_entry_in_tx(db.tx(), Object::all_reports(), ReportAction::Create)
            .await?;

        let new_report = NewReport::builder()
            .id(ReportId::new())
            .audit_info(audit_info)
            .build()
            .expect("Could not build report");

        let report = self.repo.create_in_op(&mut db, new_report).await?;

        self.jobs
            .create_and_spawn_in_op(
                &mut db,
                report.id,
                GenerateReportConfig {
                    report_id: report.id,
                },
            )
            .await?;

        Ok(JobCompletion::RescheduleAtWithOp(
            db,
            self.config.job_interval.timestamp(),
        ))
    }
}
