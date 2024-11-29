#![allow(clippy::blocks_in_conditions)]

use async_trait::async_trait;
use chrono::{DateTime, Datelike, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    audit::*,
    authorization::{Object, ReportAction},
    job::*,
    primitives::*,
    report::{repo::ReportRepo, NewReport},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateReportJobConfig {
    pub job_interval: CreateReportInterval,
}
impl JobConfig for CreateReportJobConfig {
    type Initializer = CreateReportInitializer;
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
                let d = crate::time::now();
                Utc.with_ymd_and_hms(d.year(), d.month(), d.day(), 23, 59, 59)
                    .single()
                    .expect("should return a valid date time")
            }
        }
    }
}

pub struct CreateReportInitializer {
    repo: ReportRepo,
    jobs: Jobs,
    audit: Audit,
}

impl CreateReportInitializer {
    pub fn new(repo: &ReportRepo, jobs: &Jobs, audit: &Audit) -> Self {
        Self {
            repo: repo.clone(),
            jobs: jobs.clone(),
            audit: audit.clone(),
        }
    }
}

const CREATE_REPORT_JOB: JobType = JobType::new("create-report");
impl JobInitializer for CreateReportInitializer {
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
    #[tracing::instrument(name = "lana.report.jobs.create.run", skip_all, fields(insert_id), err)]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut db = self.repo.begin_op().await?;

        let audit_info = self
            .audit
            .record_system_entry_in_tx(db.tx(), Object::Report, ReportAction::Create)
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
                super::generate::GenerateReportConfig {
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
