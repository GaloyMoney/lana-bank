use async_trait::async_trait;
use job::{
    CurrentJob, Job, JobCompletion, JobConfig, JobInitializer, JobRunner, JobType, RetrySettings,
};
use serde::{Deserialize, Serialize};

use outbox::OutboxEventMarker;

use crate::{event::CoreReportEvent, primitives::*, report::*, report_run::*};
use airflow::Airflow;

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    report_run_id: ReportRunId,
    _phantom: std::marker::PhantomData<E>,
}

impl<E> MonitorReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(report_run_id: ReportRunId) -> Self {
        Self {
            report_run_id,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E> JobConfig for MonitorReportRunJobConfig<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    type Initializer = MonitorReportRunJobInit<E>;
}

pub struct MonitorReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub airflow: Airflow,
    pub report_run_repo: ReportRunRepo<E>,
    pub report_repo: ReportRepo<E>,
}

impl<E> MonitorReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    pub fn new(
        airflow: Airflow,
        report_run_repo: ReportRunRepo<E>,
        report_repo: ReportRepo<E>,
    ) -> Self {
        Self {
            airflow,
            report_run_repo,
            report_repo,
        }
    }
}

const MONITOR_REPORT_RUN_JOB_TYPE: JobType = JobType::new("monitor-report-run");

impl<E> JobInitializer for MonitorReportRunJobInit<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    fn job_type() -> JobType {
        MONITOR_REPORT_RUN_JOB_TYPE
    }

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>> {
        let config: MonitorReportRunJobConfig<E> = job.config()?;
        Ok(Box::new(MonitorReportRunJobRunner {
            config,
            airflow: self.airflow.clone(),
            report_repo: self.report_repo.clone(),
            report_run_repo: self.report_run_repo.clone(),
        }))
    }

    fn retry_on_error_settings() -> RetrySettings {
        RetrySettings::repeat_indefinitely()
    }
}

pub struct MonitorReportRunJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent>,
{
    config: MonitorReportRunJobConfig<E>,
    airflow: Airflow,
    report_run_repo: ReportRunRepo<E>,
    report_repo: ReportRepo<E>,
}

#[async_trait]
impl<E> JobRunner for MonitorReportRunJobRunner<E>
where
    E: OutboxEventMarker<CoreReportEvent> + Send + Sync + 'static,
{
    #[tracing::instrument(
        name = "core_reports.job.monitor_report_run.run",
        skip(self, _current_job),
        err
    )]
    async fn run(
        &self,
        mut _current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>> {
        let mut report_run = self
            .report_run_repo
            .find_by_id(self.config.report_run_id)
            .await?;

        while let Some(details) = self
            .airflow
            .reports()
            .get_run(&report_run.external_id)
            .await?
        {
            if ReportRunState::from(details.state) != report_run.state {
                report_run.update_state(
                    details.state.into(),
                    details.run_type.into(),
                    details.execution_date,
                    details.start_date,
                    details.end_date,
                );
                self.report_run_repo.update(&mut report_run).await?;
            }

            if let Some(reports) = details.reports {
                for report in reports {
                    let new_report = NewReport::builder()
                        .external_id(report.id)
                        .run_id(report_run.id)
                        .name(report.name)
                        .norm(report.norm)
                        .files(report.files.into_iter().map(Into::into).collect())
                        .build()?;

                    match self.report_repo.create(new_report).await {
                        Ok(_) => {}
                        Err(e)
                            if e.to_string()
                                .contains("duplicate key value violates unique constraint") => {}
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
            }

            if matches!(
                report_run.state,
                ReportRunState::Failed | ReportRunState::Success
            ) {
                return Ok(JobCompletion::Complete);
            }
        }

        Ok(JobCompletion::RescheduleNow)
    }
}
