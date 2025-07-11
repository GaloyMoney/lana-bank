use async_trait::async_trait;
use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings,
};
use serde::{Deserialize, Serialize};

use crate::airflow::ReportsApiClient;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportDatesJobConfig;

impl JobConfig for ReportDatesJobConfig {
    type Initializer = ReportDatesJobInit;
}

pub struct ReportDatesJobInit {
    pub reports_api_client: ReportsApiClient,
}

impl ReportDatesJobInit {
    pub fn new(reports_api_client: ReportsApiClient) -> Self {
        Self { reports_api_client }
    }
}

const REPORT_DATES_JOB_TYPE: JobType = JobType::new("report-dates");

impl JobInitializer for ReportDatesJobInit {
    fn job_type() -> JobType {
        REPORT_DATES_JOB_TYPE
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let _config: ReportDatesJobConfig = job.config()?;
        Ok(Box::new(ReportDatesJobRunner {
            reports_api_client: self.reports_api_client.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct ReportDatesJobRunner {
    reports_api_client: ReportsApiClient,
}

#[async_trait]
impl JobRunner for ReportDatesJobRunner {
    #[tracing::instrument(name = "report_dates_job.run", skip(self, _current_job), err)]
    async fn run(
        &self,
        _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        println!("Starting report dates job");

        match self.reports_api_client.get_report_dates().await {
            Ok(dates) => {
                println!("Successfully retrieved report dates");
                for date in &dates {
                    println!("Available report date: {date}");
                }
                println!("Total report dates found: {}", dates.len());
                Ok(JobCompletion::RescheduleNow)
            }
            Err(e) => {
                let error_msg = format!("Failed to retrieve report dates: {e}");
                tracing::error!("{error_msg}");
                Err(Box::new(e))
            }
        }
    }
}
