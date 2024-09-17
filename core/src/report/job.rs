#![allow(clippy::blocks_in_conditions)]
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    audit::*,
    authorization::{Object, ReportAction},
    job::*,
    primitives::*,
};

use super::{
    dataform_client::DataformClient, entity::ReportGenerationProcessStep, repo::ReportRepo,
    ReportConfig,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReportConfig {
    pub(super) report_id: ReportId,
}

pub struct GenerateReportInitializer {
    repo: ReportRepo,
    report_config: ReportConfig,
    audit: Audit,
}

impl GenerateReportInitializer {
    pub fn new(repo: &ReportRepo, report_config: &ReportConfig, audit: &Audit) -> Self {
        Self {
            repo: repo.clone(),
            report_config: report_config.clone(),
            audit: audit.clone(),
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
        }))
    }
}

pub struct GenerateReportJobRunner {
    config: GenerateReportConfig,
    repo: ReportRepo,
    report_config: ReportConfig,
    audit: Audit,
}

#[async_trait]
impl JobRunner for GenerateReportJobRunner {
    #[tracing::instrument(name = "lava.report.job.run", skip_all, fields(insert_id), err)]
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut report = self.repo.find_by_id(self.config.report_id).await?;
        let mut client = DataformClient::connect(&self.report_config).await?;

        match report.next_step() {
            ReportGenerationProcessStep::Compilation => {
                let mut db_tx = current_job.pool().begin().await?;

                let audit_info = self
                    .audit
                    .record_entry_in_tx(
                        &mut db_tx,
                        &Subject::System(SystemNode::Core),
                        Object::Report,
                        ReportAction::Compile,
                        true,
                    )
                    .await?;
                match client.compile().await {
                    Ok(res) => {
                        report.compilation_completed(res, audit_info);
                    }
                    Err(e) => {
                        report.compilation_failed(e.to_string(), audit_info);
                    }
                }
                self.repo.persist_in_tx(&mut db_tx, &mut report).await?;
                db_tx.commit().await?;

                return Ok(JobCompletion::RescheduleAt(chrono::Utc::now()));
            }

            ReportGenerationProcessStep::Invocation => {
                let mut db_tx = current_job.pool().begin().await?;

                let audit_info = self
                    .audit
                    .record_entry_in_tx(
                        &mut db_tx,
                        &Subject::System(SystemNode::Core),
                        Object::Report,
                        ReportAction::Invoke,
                        true,
                    )
                    .await?;
                match client.invoke(&report.compilation_result()).await {
                    Ok(res) => {
                        report.invocation_completed(res, audit_info);
                    }
                    Err(e) => {
                        report.invocation_failed(e.to_string(), audit_info);
                    }
                }
                self.repo.persist_in_tx(&mut db_tx, &mut report).await?;
                db_tx.commit().await?;

                return Ok(JobCompletion::RescheduleAt(chrono::Utc::now()));
            }

            ReportGenerationProcessStep::Upload => {
                let mut db_tx = current_job.pool().begin().await?;

                let audit_info = self
                    .audit
                    .record_entry_in_tx(
                        &mut db_tx,
                        &Subject::System(SystemNode::Core),
                        Object::Report,
                        ReportAction::Upload,
                        true,
                    )
                    .await?;

                match super::upload::execute(&self.report_config).await {
                    Ok(res) => {
                        report.upload_completed(res, audit_info);
                    }
                    Err(e) => {
                        report.upload_failed(e.to_string(), audit_info);

                        self.repo.persist_in_tx(&mut db_tx, &mut report).await?;
                        db_tx.commit().await?;

                        return Ok(JobCompletion::RescheduleAt(chrono::Utc::now()));
                    }
                }

                self.repo.persist_in_tx(&mut db_tx, &mut report).await?;
                db_tx.commit().await?;
            }
        }

        Ok(JobCompletion::Complete)
    }
}
