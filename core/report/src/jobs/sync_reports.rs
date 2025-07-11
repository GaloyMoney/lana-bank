use async_trait::async_trait;
use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings,
};
use serde::{Deserialize, Serialize};

use crate::airflow::ReportsApiClient;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncReportsJobConfig;

impl JobConfig for SyncReportsJobConfig {
    type Initializer = SyncReportsJobInit;
}

pub struct SyncReportsJobInit {
    pub reports_api_client: ReportsApiClient,
}

impl SyncReportsJobInit {
    pub fn new(reports_api_client: ReportsApiClient) -> Self {
        Self { reports_api_client }
    }
}

const SYNC_REPORTS_JOB_TYPE: JobType = JobType::new("sync-reports");

impl JobInitializer for SyncReportsJobInit {
    fn job_type() -> JobType {
        SYNC_REPORTS_JOB_TYPE
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let _config: SyncReportsJobConfig = job.config()?;
        Ok(Box::new(SyncReportsJobRunner {
            reports_api_client: self.reports_api_client.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct SyncReportsJobRunner {
    reports_api_client: ReportsApiClient,
}

#[async_trait]
impl JobRunner for SyncReportsJobRunner {
    #[tracing::instrument(name = "sync_reports_job.run", skip(self, _current_job), err)]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        println!("Starting sync reports job");

        match self.reports_api_client.get_report_dates().await {
            Ok(dates) => {
                println!("Successfully synced reports");
                for date in &dates {
                    println!("Available report date: {}", date.format("%Y-%m-%d"));
                }
                println!("Total reports synced: {}", dates.len());
                Ok(JobCompletion::RescheduleNow)
            }
            Err(e) => {
                let error_msg = format!("Failed to sync reports: {e}");
                tracing::error!("{error_msg}");
                Err(Box::new(e))
            }
        }
    }
}
