use serde_json::Value as JsonValue;
use sqlx::postgres::{PgListener, PgPool, types::PgInterval};
use tracing::{Span, instrument};

use std::{sync::Arc, time::Duration};

use super::{
    JobId, config::JobsConfig, dispatcher::*, error::JobError, handle::OwnedTaskHandle,
    registry::JobRegistry, repo::JobRepo, tracker::JobTracker,
};

pub(crate) struct JobPoller {
    config: JobsConfig,
    repo: JobRepo,
    registry: JobRegistry,
    tracker: Arc<JobTracker>,
}

#[allow(dead_code)]
pub(crate) struct JobPollerHandle {
    poller: Arc<JobPoller>,
    handle: OwnedTaskHandle,
}

const MAX_WAIT: Duration = Duration::from_secs(60);

impl JobPoller {
    pub fn new(config: JobsConfig, repo: JobRepo, registry: JobRegistry) -> Self {
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

    pub async fn start(self) -> Result<JobPollerHandle, sqlx::Error> {
        let listener_handle = self.start_listener().await?;
        let lost_handle = self.start_lost_handler();
        let executor = Arc::new(self);
        let handle = OwnedTaskHandle::new(tokio::task::spawn(Self::main_loop(
            Arc::clone(&executor),
            listener_handle,
            lost_handle,
        )));
        Ok(JobPollerHandle {
            poller: executor,
            handle,
        })
    }

    async fn main_loop(
        self: Arc<Self>,
        _listener_task: OwnedTaskHandle,
        _lost_task: OwnedTaskHandle,
    ) {
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
        fields(n_jobs_running, n_jobs_to_start, now, next_poll_in),
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
                span.record("next_poll_in", tracing::field::debug(duration));
                return Ok(duration);
            }
            JobPollResult::Jobs(jobs) => jobs,
        };
        span.record("n_jobs_to_start", rows.len());
        if !rows.is_empty() {
            for row in rows {
                self.dispatch_job(row).await?;
            }
        }

        span.record("next_poll_in", tracing::field::debug(Duration::ZERO));
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

    fn start_lost_handler(&self) -> OwnedTaskHandle {
        let job_lost_interval = self.config.job_lost_interval;
        let pool = self.repo.pool().clone();
        OwnedTaskHandle::new(tokio::task::spawn(async move {
            loop {
                crate::time::sleep(job_lost_interval / 2).await;
                let check_time = crate::time::now() - job_lost_interval;
                let _ = sqlx::query!(
                    r#"
            UPDATE job_executions
            SET state = 'pending', execute_at = $1, attempt_index = attempt_index + 1
            WHERE state = 'running' AND alive_at < $1::timestamptz
            "#,
                    check_time,
                )
                .fetch_all(&pool)
                .await;
            }
        }))
    }

    #[instrument(
        name = "job.dispatch_job",
        skip(self),
        fields(job_id, job_type, attempt, now),
        err
    )]
    async fn dispatch_job(&self, polled_job: PolledJob) -> Result<(), JobError> {
        let span = Span::current();
        span.record("attempt", polled_job.attempt);
        let job = self.repo.find_by_id(polled_job.id).await?;
        span.record("job_id", tracing::field::display(job.id));
        span.record("job_type", tracing::field::display(&job.job_type));
        let runner = self.registry.init_job(&job)?;
        let retry_settings = self.registry.retry_settings(&job.job_type).clone();
        let repo = self.repo.clone();
        let tracker = self.tracker.clone();
        let job_lost_interval = self.config.job_lost_interval;
        span.record("now", tracing::field::display(crate::time::now()));
        tokio::spawn(async move {
            let _ = JobDispatcher::new(
                repo,
                tracker,
                retry_settings,
                job.id,
                runner,
                job_lost_interval,
            )
            .execute_job(polled_job)
            .await;
        });
        Ok(())
    }
}

async fn poll_jobs(pool: &PgPool, n_jobs_to_poll: usize) -> Result<JobPollResult, sqlx::Error> {
    let now = crate::time::now();
    Span::current().record("now", tracing::field::display(now));

    // Debug: Show all job_executions rows
    let debug_rows = sqlx::query!(
        r#"
        SELECT 
            id,
            state as "state: String",
            execute_at,
            job_type,
            attempt_index,
            CASE 
                WHEN execute_at IS NULL THEN 'NULL'
                WHEN execute_at <= $1 THEN 'READY (' || (execute_at::text) || ')'
                ELSE 'FUTURE (' || (execute_at::text) || ')'
            END as "execute_status: String",
            execute_at - $1 as "time_diff: PgInterval"
        FROM job_executions
        ORDER BY execute_at ASC NULLS LAST
        "#,
        now
    )
    .fetch_all(pool)
    .await?;

    eprintln!("=== DEBUG: All job_executions rows ===");
    eprintln!("Current time (now): {:?}", now);
    eprintln!("Number of rows: {}", debug_rows.len());
    for row in &debug_rows {
        eprintln!(
            "ID: {:?}, State: {}, JobType: {}, Attempt: {}, ExecuteAt: {:?}, Status: {:?}, Diff: {:?}",
            row.id,
            row.state,
            row.job_type,
            row.attempt_index,
            row.execute_at,
            row.execute_status,
            row.time_diff
        );
    }
    eprintln!("=== END DEBUG ===");

    let rows = sqlx::query_as!(
        JobPollRow,
        r#"
        WITH min_wait AS (
            SELECT MIN(execute_at) - $2::timestamptz AS wait_time
            FROM job_executions
            WHERE state = 'pending'
            AND execute_at > $2::timestamptz
        ),
        selected_jobs AS (
            SELECT je.id, je.execution_state_json AS data_json, je.job_type, je.attempt_index
            FROM job_executions je
            JOIN jobs ON je.id = jobs.id
            WHERE execute_at <= $2::timestamptz
            AND je.state = 'pending'
            ORDER BY execute_at ASC
            LIMIT $1
            FOR UPDATE
        ),
        updated AS (
            UPDATE job_executions AS je
            SET state = 'running', execute_at = NULL
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
enum JobPollResult {
    Jobs(Vec<PolledJob>),
    WaitTillNextJob(Duration),
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
                            attempt: attempt_index as u32,
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
