use serde_json::Value as JsonValue;
use sqlx::postgres::{PgListener, PgPool, types::PgInterval};
use tracing::{Span, instrument};

use std::{sync::Arc, time::Duration};

use super::{
    JobId, config::JobExecutorConfig, entity::Job, error::JobError, handle::OwnedTaskHandle,
    registry::JobRegistry, repo::JobRepo, tracker::JobTracker,
};

pub(crate) struct NewJobExecutor {
    config: JobExecutorConfig,
    repo: JobRepo,
    registry: JobRegistry,
    tracker: Arc<JobTracker>,
}

pub(crate) struct JobExecutorHandle {
    executor: Arc<NewJobExecutor>,
    handle: OwnedTaskHandle,
}

const MAX_WAIT: Duration = Duration::from_secs(60);

impl NewJobExecutor {
    pub fn new(config: JobExecutorConfig, repo: JobRepo, registry: JobRegistry) -> Self {
        Self {
            tracker: Arc::new(JobTracker::new(
                config.min_jobs_per_process,
                config.max_jobs_per_process,
            )),
            repo,
            config,
            registry,
        }
    }

    pub async fn start(self) -> Result<JobExecutorHandle, sqlx::Error> {
        let listener_handle = self.start_listener().await?;
        let executor = Arc::new(self);
        let handle = OwnedTaskHandle::new(tokio::task::spawn(Self::main_loop(
            Arc::clone(&executor),
            listener_handle,
        )));
        Ok(JobExecutorHandle { executor, handle })
    }

    async fn main_loop(self: Arc<Self>, _listener_task: OwnedTaskHandle) {
        let mut failures = 0;
        loop {
            let mut timeout = MAX_WAIT;
            if let Some(batch_size) = self.tracker.next_batch_size() {
                match self.poll_and_dispatch(batch_size).await {
                    Ok(duration) => {
                        timeout = duration;
                        failures = 0;
                    }
                    Err(_) => {
                        failures += 1;
                        timeout = Duration::from_millis(50 << failures);
                    }
                }
            };
            let _ = crate::time::timeout(timeout, self.tracker.notified()).await;
        }
    }

    #[instrument(
        name = "job.poll_and_dispatch",
        skip(self),
        fields(n_jobs_running, n_jobs_to_start, jobs_to_start),
        err
    )]
    async fn poll_and_dispatch(
        self: &Arc<Self>,
        n_jobs_to_poll: usize,
    ) -> Result<Duration, JobError> {
        let span = Span::current();
        self.tracker.trace_n_jobs_running();
        let rows = match poll_jobs(self.repo.pool(), n_jobs_to_poll).await? {
            JobPollResult::WaitTillNextJob(duration) => {
                return Ok(duration);
            }
            JobPollResult::Jobs(jobs) => jobs,
        };
        span.record("n_jobs_to_start", rows.len());
        span.record(
            "jobs_to_start",
            rows.iter()
                .map(|r| r.job_type.as_str())
                .collect::<Vec<_>>()
                .join(","),
        );
        if !rows.is_empty() {
            for row in rows {
                let executor = Arc::clone(self);
                tokio::spawn(async move {
                    let _ = executor.execute_job(row).await;
                });
            }
        }

        Ok(Duration::ZERO)
    }

    async fn start_listener(&self) -> Result<OwnedTaskHandle, sqlx::Error> {
        let mut listener = PgListener::connect_with(self.repo.pool()).await?;
        listener.listen("job_execution").await?;
        let tracker = self.tracker.clone();
        Ok(OwnedTaskHandle::new(tokio::task::spawn(async move {
            loop {
                if listener.recv().await.is_ok() {
                    tracker.job_execution_inserted();
                } else {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        })))
    }

    #[instrument(name = "job.execute_job", skip_all,
        fields(job_id, job_type, attempt, error, error.level, error.message, conclusion),
    err)]
    async fn execute_job(self: Arc<Self>, polled_job: PolledJob) -> Result<(), JobError> {
        let job = self.repo.find_by_id(polled_job.id).await?;
        let runner = self.registry.init_job(&job)?;
        let span = Span::current();
        span.record("job_id", tracing::field::display(job.id));
        span.record("job_type", tracing::field::display(&job.job_type));
        span.record("attempt", polled_job.attempt_index);
        Ok(())
    }
}

async fn poll_jobs(pool: &PgPool, n_jobs_to_poll: usize) -> Result<JobPollResult, sqlx::Error> {
    let now = crate::time::now();
    let rows = sqlx::query_as!(
        JobPollRow,
        r#"
        WITH min_wait AS (
            SELECT MIN(reschedule_after) - $2::timestamptz AS wait_time
            FROM job_executions
            WHERE state = 'pending'
            AND reschedule_after > $2::timestamptz
        ),
        selected_jobs AS (
            SELECT je.id, je.execution_state_json AS data_json, je.job_type, je.attempt_index
            FROM job_executions je
            JOIN jobs ON je.id = jobs.id
            WHERE reschedule_after < $2::timestamptz
            AND je.state = 'pending'
            ORDER BY reschedule_after ASC
            LIMIT $1
            FOR UPDATE
        ),
        updated AS (
            UPDATE job_executions AS je
            SET state = 'running', reschedule_after = NULL
            FROM selected_jobs
            WHERE je.id = selected_jobs.id
            RETURNING je.id, je.job_type, selected_jobs.data_json, je.attempt_index
        )
        SELECT * FROM (
            SELECT 
                u.id AS "id?: JobId",
                u.job_type AS "job_type?",
                u.data_json AS "data_json?: JsonValue",
                u.attempt_index AS "attempt_index?",
                NULL::INTERVAL AS "max_wait?: PgInterval"
            FROM updated u
            UNION ALL
            SELECT 
                NULL::UUID AS "id?: JobId",
                NULL::VARCHAR AS "job_type?",
                NULL::JSONB AS "data_json?: JsonValue",
                NULL::INT AS "attempt_index?",
                mw.wait_time AS "max_wait?: PgInterval"
            FROM min_wait mw
            WHERE NOT EXISTS (SELECT 1 FROM updated)
        ) AS result
        "#,
        n_jobs_to_poll as i32,
        now,
    )
    .fetch_all(pool)
    .await?;

    Ok(JobPollResult::from_rows(rows))
}

#[derive(Debug)]
pub enum JobPollResult {
    Jobs(Vec<PolledJob>),
    WaitTillNextJob(Duration),
}

#[derive(Debug)]
pub struct PolledJob {
    pub id: JobId,
    pub job_type: String,
    pub data_json: Option<JsonValue>,
    pub attempt_index: i32,
}

#[derive(Debug)]
struct JobPollRow {
    id: Option<JobId>,
    job_type: Option<String>,
    data_json: Option<JsonValue>,
    attempt_index: Option<i32>,
    max_wait: Option<PgInterval>,
}

impl JobPollResult {
    /// Convert raw query rows into a JobPollResult
    pub fn from_rows(rows: Vec<JobPollRow>) -> Self {
        if rows.is_empty() {
            JobPollResult::WaitTillNextJob(MAX_WAIT)
        } else if rows.len() == 1 && rows[0].id.is_none() {
            if let Some(interval) = &rows[0].max_wait {
                JobPollResult::WaitTillNextJob(pg_interval_to_duration(interval))
            } else {
                JobPollResult::WaitTillNextJob(MAX_WAIT)
            }
        } else {
            let jobs = rows
                .into_iter()
                .filter_map(|row| {
                    if let (Some(id), Some(job_type), Some(attempt_index)) =
                        (row.id, row.job_type, row.attempt_index)
                    {
                        Some(PolledJob {
                            id,
                            job_type,
                            data_json: row.data_json,
                            attempt_index,
                        })
                    } else {
                        None
                    }
                })
                .collect();
            JobPollResult::Jobs(jobs)
        }
    }
}

fn pg_interval_to_duration(interval: &PgInterval) -> Duration {
    const SECONDS_PER_DAY: u64 = 24 * 60 * 60;
    if interval.microseconds < 0 || interval.days < 0 || interval.months < 0 {
        Duration::default()
    } else {
        let days = (interval.days as u64) + (interval.months as u64) * 30;
        Duration::from_micros(interval.microseconds as u64)
            + Duration::from_secs(days * SECONDS_PER_DAY)
    }
}
