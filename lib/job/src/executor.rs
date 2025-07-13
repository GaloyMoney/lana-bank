use chrono::{DateTime, Utc};
use sqlx::postgres::{PgListener, PgPool, types::PgInterval};
use tokio::sync::{Notify, RwLock};
use tracing::{Span, instrument};

use std::{collections::HashMap, sync::Arc, time::Duration};

use super::{
    JobId, config::*, current::*, entity::*, error::JobError, handle::*, registry::*, repo::*,
    traits::*,
};

#[derive(Clone)]
pub(crate) struct JobExecutor {
    config: JobExecutorConfig,
    poll_handle: Option<Arc<OwnedTaskHandle>>,
    keep_alive_handle: Option<Arc<OwnedTaskHandle>>,
    listen_handle: Option<Arc<OwnedTaskHandle>>,
    running_jobs: Arc<RwLock<HashMap<JobId, OwnedTaskHandle>>>,
    notify: Arc<Notify>,
    jobs: JobRepo,
}

impl JobExecutor {
    pub fn new(config: JobExecutorConfig, jobs: &JobRepo) -> Self {
        Self {
            poll_handle: None,
            keep_alive_handle: None,
            listen_handle: None,
            config,
            running_jobs: Arc::new(RwLock::new(HashMap::new())),
            notify: Arc::new(Notify::new()),
            jobs: jobs.clone(),
        }
    }

    pub async fn start(&mut self, registry: &Arc<RwLock<JobRegistry>>) -> Result<(), JobError> {
        let keep_alive_interval = self.config.keep_alive_interval;
        let max_concurrency = self.config.max_jobs_per_process;
        let min_concurrency = self.config.min_jobs_per_process;
        let pg_interval = PgInterval::try_from(keep_alive_interval * 4)
            .map_err(|e| JobError::InvalidPollInterval(e.to_string()))?;
        let running_jobs = Arc::clone(&self.running_jobs);
        let registry = Arc::clone(registry);
        let jobs = self.jobs.clone();

        // Spawn keep_alive thread
        let keep_alive_jobs = Arc::clone(&running_jobs);
        let keep_alive_repo = jobs.clone();
        let keep_alive_pg_interval = pg_interval;
        let keep_alive_handle = tokio::spawn(async move {
            loop {
                let _ = Self::keep_alive_jobs(
                    &keep_alive_jobs,
                    &keep_alive_repo,
                    keep_alive_pg_interval,
                )
                .await;
                crate::time::sleep(keep_alive_interval).await;
            }
        });
        self.keep_alive_handle = Some(Arc::new(OwnedTaskHandle::new(keep_alive_handle)));

        let notify = self.notify.clone();
        let poll_handle = tokio::spawn(async move {
            let mut failures = 0;
            loop {
                let max_wait = if Self::poll_jobs(
                    &registry,
                    max_concurrency,
                    min_concurrency,
                    pg_interval,
                    &running_jobs,
                    &jobs,
                    &notify,
                )
                .await
                .is_err()
                {
                    failures += 1;
                    Duration::from_millis(50 << failures)
                } else {
                    failures = 0;
                    Duration::from_secs(60)
                };
                let _ = crate::time::timeout(max_wait, notify.notified()).await;
            }
        });
        self.poll_handle = Some(Arc::new(OwnedTaskHandle::new(poll_handle)));

        let listen_handle = start_listener(self.jobs.pool(), self.notify.clone()).await?;
        self.listen_handle = Some(Arc::new(listen_handle));
        Ok(())
    }

    #[instrument(
        name = "job.keep_alive_jobs",
        skip(running_jobs, jobs),
        fields(n_jobs_running),
        err
    )]
    async fn keep_alive_jobs(
        running_jobs: &Arc<RwLock<HashMap<JobId, OwnedTaskHandle>>>,
        jobs: &JobRepo,
        pg_interval: PgInterval,
    ) -> Result<(), JobError> {
        let span = Span::current();
        let now = crate::time::now();
        let running_jobs_read = running_jobs.read().await;
        let n_jobs_running = running_jobs_read.len();
        span.record("n_jobs_running", n_jobs_running);

        if n_jobs_running > 0 {
            let ids = running_jobs_read.keys().cloned().collect::<Vec<_>>();
            sqlx::query!(
                r#"
                UPDATE job_executions
                SET reschedule_after = $2::timestamptz + $3::interval
                WHERE id = ANY($1)
                "#,
                &ids as &[JobId],
                now,
                pg_interval
            )
            .fetch_all(jobs.pool())
            .await?;
        }
        drop(running_jobs_read);

        // mark 'lost' jobs as 'pending'
        sqlx::query!(
            r#"
            UPDATE job_executions
            SET state = 'pending', attempt_index = attempt_index + 1
            WHERE state = 'running' AND reschedule_after < $1::timestamptz + $2::interval
            "#,
            now,
            pg_interval
        )
        .fetch_all(jobs.pool())
        .await?;

        Ok(())
    }

    #[instrument(
        name = "job.poll_jobs",
        skip(registry, running_jobs, jobs),
        fields(n_jobs_running, n_jobs_to_poll, n_jobs_to_start, jobs_to_start),
        err
    )]
    async fn poll_jobs(
        registry: &Arc<RwLock<JobRegistry>>,
        max_concurrency: usize,
        min_concurrency: usize,
        pg_interval: PgInterval,
        running_jobs: &Arc<RwLock<HashMap<JobId, OwnedTaskHandle>>>,
        jobs: &JobRepo,
        notify: &Arc<Notify>,
    ) -> Result<(), JobError> {
        let span = Span::current();
        let now = crate::time::now();
        let n_jobs_running = {
            let running_jobs = running_jobs.read().await;
            let n_jobs_running = running_jobs.len();
            span.record("n_jobs_running", n_jobs_running);
            n_jobs_running
        };

        if n_jobs_running > min_concurrency {
            span.record("n_jobs_to_poll", 0);
            return Ok(());
        }

        let n_jobs_to_poll = max_concurrency - n_jobs_running;
        span.record("n_jobs_to_poll", n_jobs_to_poll);

        let rows = sqlx::query!(
            r#"
              WITH selected_jobs AS (
                  SELECT je.id, je.execution_state_json AS data_json
                  FROM job_executions je
                  JOIN jobs ON je.id = jobs.id
                  WHERE reschedule_after < $2::timestamptz
                  AND je.state = 'pending'
                  ORDER BY reschedule_after ASC
                  LIMIT $1
                  FOR UPDATE
              )
              UPDATE job_executions AS je
              SET state = 'running', reschedule_after = $2::timestamptz + $3::interval
              FROM selected_jobs
              WHERE je.id = selected_jobs.id
              RETURNING je.id AS "id!: JobId", je.job_type, selected_jobs.data_json, je.attempt_index
              "#,
            n_jobs_to_poll as i32,
            now,
            pg_interval
        )
        .fetch_all(jobs.pool())
        .await?;
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
                let job = jobs.find_by_id(row.id).await?;
                let _ = Self::start_job(
                    registry,
                    running_jobs,
                    job,
                    row.attempt_index as u32,
                    row.data_json,
                    jobs.clone(),
                    notify,
                )
                .await;
            }
        }
        Ok(())
    }

    #[instrument(
        name = "job.start_job",
        skip(registry, running_jobs, job, repo),
        fields(job_id, job_type),
        err
    )]
    async fn start_job(
        registry: &Arc<RwLock<JobRegistry>>,
        running_jobs: &Arc<RwLock<HashMap<JobId, OwnedTaskHandle>>>,
        job: Job,
        attempt: u32,
        job_payload: Option<serde_json::Value>,
        repo: JobRepo,
        notify: &Arc<Notify>,
    ) -> Result<(), JobError> {
        let runner = registry
            .try_read()
            .expect("cannot read registry")
            .init_job(&job)?;
        let id = job.id;
        let span = Span::current();
        span.record("job_id", tracing::field::display(&id));
        span.record("job_type", tracing::field::display(&job.job_type));
        let job_type = job.job_type.clone();
        let all_jobs = Arc::clone(running_jobs);
        let registry = Arc::clone(registry);
        let notify = Arc::clone(notify);
        let handle = tokio::spawn(async move {
            let res =
                Self::execute_job(job, attempt, job_payload, runner, repo.clone(), &registry).await;
            let mut write_lock = all_jobs.write().await;
            if let Err(e) = res {
                match repo.begin_op().await {
                    Ok(op) => {
                        let _ = Self::fail_job(
                            op,
                            id,
                            attempt,
                            e,
                            repo,
                            registry
                                .try_read()
                                .expect("Cannot read registry")
                                .retry_settings(&job_type),
                        )
                        .await;
                    }
                    Err(_) => {
                        eprintln!("Could not start transaction when failing job");
                        tracing::error!("Could not start transaction when failing job");
                    }
                }
            }
            write_lock.remove(&id);
            notify.notify_one();
        });
        running_jobs
            .write()
            .await
            .insert(id, OwnedTaskHandle::new(handle));
        Ok(())
    }

    #[instrument(name = "job.execute_job", skip_all,
        fields(job_id, job_type, attempt, error, error.level, error.message, conclusion),
    err)]
    async fn execute_job(
        job: Job,
        attempt: u32,
        payload: Option<serde_json::Value>,
        runner: Box<dyn JobRunner>,
        repo: JobRepo,
        registry: &Arc<RwLock<JobRegistry>>,
    ) -> Result<(), JobError> {
        let id = job.id;
        let span = Span::current();
        span.record("job_id", tracing::field::display(&id));
        span.record("job_type", tracing::field::display(&job.job_type));
        span.record("attempt", attempt);
        let current_job_pool = repo.pool().clone();
        let current_job = CurrentJob::new(id, attempt, current_job_pool, payload);

        match runner.run(current_job).await.map_err(|e| {
            let error = e.to_string();
            Span::current().record("error", tracing::field::display("true"));
            Span::current().record("error.message", tracing::field::display(&error));
            let n_warn_attempts = registry
                .try_read()
                .expect("Cannot read registry")
                .retry_settings(&job.job_type)
                .n_warn_attempts;
            if attempt <= n_warn_attempts.unwrap_or(u32::MAX) {
                Span::current()
                    .record("error.level", tracing::field::display(tracing::Level::WARN));
            } else {
                Span::current().record(
                    "error.level",
                    tracing::field::display(tracing::Level::ERROR),
                );
            }
            JobError::JobExecutionError(error)
        })? {
            JobCompletion::Complete => {
                span.record("conclusion", "Complete");
                let op = repo.begin_op().await?;
                Self::complete_job(op, id, repo).await?;
            }
            JobCompletion::CompleteWithOp(op) => {
                span.record("conclusion", "CompleteWithOp");
                Self::complete_job(op, id, repo).await?;
            }
            JobCompletion::RescheduleNow => {
                span.record("conclusion", "RescheduleNow");
                let op = repo.begin_op().await?;
                let t = op.now();
                Self::reschedule_job(op, id, t).await?;
            }
            JobCompletion::RescheduleNowWithOp(op) => {
                span.record("conclusion", "RescheduleNowWithOp");
                let t = op.now();
                Self::reschedule_job(op, id, t).await?;
            }
            JobCompletion::RescheduleIn(d) => {
                span.record("conclusion", "RescheduleIn");
                let op = repo.begin_op().await?;
                let t = op.now() + d;
                Self::reschedule_job(op, id, t).await?;
            }
            JobCompletion::RescheduleInWithOp(d, op) => {
                span.record("conclusion", "RescheduleInWithOp");
                let t = op.now() + d;
                Self::reschedule_job(op, id, t).await?;
            }
            JobCompletion::RescheduleAt(t) => {
                span.record("conclusion", "RescheduleAt");
                let op = repo.begin_op().await?;
                Self::reschedule_job(op, id, t).await?;
            }
            JobCompletion::RescheduleAtWithOp(op, t) => {
                span.record("conclusion", "RescheduleAtWithOp");
                Self::reschedule_job(op, id, t).await?;
            }
        }
        Ok(())
    }

    async fn complete_job(
        mut op: es_entity::DbOp<'_>,
        id: JobId,
        repo: JobRepo,
    ) -> Result<(), JobError> {
        let mut job = repo.find_by_id(&id).await?;
        sqlx::query!(
            r#"
          DELETE FROM job_executions
          WHERE id = $1
        "#,
            id as JobId
        )
        .execute(&mut **op.tx())
        .await?;
        job.completed();
        repo.update_in_op(&mut op, &mut job).await?;
        op.commit().await?;
        Ok(())
    }

    async fn reschedule_job(
        mut op: es_entity::DbOp<'_>,
        id: JobId,
        reschedule_at: DateTime<Utc>,
    ) -> Result<(), JobError> {
        sqlx::query!(
            r#"
          UPDATE job_executions
          SET state = 'pending', reschedule_after = $2, attempt_index = 1
          WHERE id = $1
        "#,
            id as JobId,
            reschedule_at,
        )
        .execute(&mut **op.tx())
        .await?;
        op.commit().await?;
        Ok(())
    }

    #[instrument(name = "job.fail_job", skip(op, repo))]
    async fn fail_job(
        mut op: es_entity::DbOp<'_>,
        id: JobId,
        attempt: u32,
        error: JobError,
        repo: JobRepo,
        retry_settings: &RetrySettings,
    ) -> Result<(), JobError> {
        let mut job = repo.find_by_id(id).await?;
        job.fail(error.to_string());
        repo.update_in_op(&mut op, &mut job).await?;

        if retry_settings.n_attempts.unwrap_or(u32::MAX) > attempt {
            let reschedule_at = retry_settings.next_attempt_at(attempt);
            sqlx::query!(
                r#"
                UPDATE job_executions
                SET state = 'pending', reschedule_after = $2, attempt_index = $3
                WHERE id = $1
              "#,
                id as JobId,
                reschedule_at,
                (attempt + 1) as i32
            )
            .execute(&mut **op.tx())
            .await?;
        } else {
            sqlx::query!(
                r#"
                DELETE FROM job_executions
                WHERE id = $1
              "#,
                id as JobId
            )
            .execute(&mut **op.tx())
            .await?;
        }

        op.commit().await?;
        Ok(())
    }
}

async fn start_listener(
    pool: &PgPool,
    notify: Arc<Notify>,
) -> Result<OwnedTaskHandle, sqlx::Error> {
    let mut listener = PgListener::connect_with(pool).await?;
    listener.listen("job_execution").await?;
    Ok(OwnedTaskHandle::new(tokio::task::spawn(async move {
        let mut num_errors = 0;
        loop {
            if num_errors > 0 || listener.recv().await.is_ok() {
                notify.notify_one();
                num_errors = 0;
            } else {
                tokio::time::sleep(Duration::from_secs(1 << num_errors)).await;
                num_errors += 1;
            }
        }
    })))
}
