use chrono::{DateTime, Utc};
use es_entity::AtomicOperation;
use futures::FutureExt;
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use tracing::{Span, instrument};

use std::{panic::AssertUnwindSafe, sync::Arc, time::Duration};

use super::{
    JobId, current::CurrentJob, error::JobError, handle::OwnedTaskHandle, repo::JobRepo,
    tracker::JobTracker, traits::*,
};

#[derive(Debug)]
pub struct PolledJob {
    pub id: JobId,
    #[allow(dead_code)]
    pub job_type: String,
    pub data_json: Option<JsonValue>,
    pub attempt: u32,
}

pub(crate) struct JobDispatcher {
    repo: JobRepo,
    retry_settings: RetrySettings,
    runner: Option<Box<dyn JobRunner>>,
    tracker: Arc<JobTracker>,
    keep_alive: Option<OwnedTaskHandle>,
    rescheduled: bool,
}
impl JobDispatcher {
    pub fn new(
        repo: JobRepo,
        tracker: Arc<JobTracker>,
        retry_settings: RetrySettings,
        id: JobId,
        runner: Box<dyn JobRunner>,
        job_lost_interval: Duration,
    ) -> Self {
        let keep_alive = Some(OwnedTaskHandle::new(tokio::task::spawn(keep_job_alive(
            repo.pool().clone(),
            id,
            job_lost_interval,
        ))));
        Self {
            repo,
            retry_settings,
            runner: Some(runner),
            tracker,
            keep_alive,
            rescheduled: false,
        }
    }

    #[instrument(name = "job.execute_job", skip_all,
        fields(job_id, job_type, attempt, error, error.level, error.message, conclusion, now),
    err)]
    pub async fn execute_job(mut self, polled_job: PolledJob) -> Result<(), JobError> {
        let job = self.repo.find_by_id(polled_job.id).await?;
        let span = Span::current();
        span.record("job_id", tracing::field::display(job.id));
        span.record("job_type", tracing::field::display(&job.job_type));
        span.record("attempt", polled_job.attempt);
        span.record("now", tracing::field::display(crate::time::now()));
        let current_job = CurrentJob::new(
            polled_job.id,
            polled_job.attempt,
            self.repo.pool().clone(),
            polled_job.data_json,
        );
        self.tracker.dispatch_job();
        match Self::dispatch_job(
            self.runner.take().expect("runner"),
            current_job,
            self.retry_settings.n_warn_attempts,
            polled_job.attempt,
        )
        .await
        {
            Err(e) => {
                span.record("conclusion", "Error");
                self.fail_job(job.id, e, polled_job.attempt).await?
            }
            Ok(JobCompletion::Complete) => {
                span.record("conclusion", "Complete");
                let op = self.repo.begin_op().await?;
                self.complete_job(op, job.id).await?;
            }
            Ok(JobCompletion::CompleteWithOp(op)) => {
                span.record("conclusion", "CompleteWithOp");
                self.complete_job(op, job.id).await?;
            }
            Ok(JobCompletion::RescheduleNow) => {
                span.record("conclusion", "RescheduleNow");
                let op = self.repo.begin_op().await?;
                let t = op.now().unwrap_or_else(crate::time::now);
                self.reschedule_job(op, job.id, t).await?;
            }
            Ok(JobCompletion::RescheduleNowWithOp(op)) => {
                span.record("conclusion", "RescheduleNowWithOp");
                let t = op.now().unwrap_or_else(crate::time::now);
                self.reschedule_job(op, job.id, t).await?;
            }
            Ok(JobCompletion::RescheduleIn(d)) => {
                span.record("conclusion", "RescheduleIn");
                let op = self.repo.begin_op().await?;
                let t = op.now().unwrap_or_else(crate::time::now);
                let t = t + d;
                self.reschedule_job(op, job.id, t).await?;
            }
            Ok(JobCompletion::RescheduleInWithOp(d, op)) => {
                span.record("conclusion", "RescheduleInWithOp");
                let t = op.now().unwrap_or_else(crate::time::now);
                let t = t + d;
                self.reschedule_job(op, job.id, t).await?;
            }
            Ok(JobCompletion::RescheduleAt(t)) => {
                span.record("conclusion", "RescheduleAt");
                let op = self.repo.begin_op().await?;
                self.reschedule_job(op, job.id, t).await?;
            }
            Ok(JobCompletion::RescheduleAtWithOp(op, t)) => {
                span.record("conclusion", "RescheduleAtWithOp");
                self.reschedule_job(op, job.id, t).await?;
            }
        }
        Ok(())
    }

    async fn dispatch_job(
        runner: Box<dyn JobRunner>,
        current_job: CurrentJob,
        n_warn_attempts: Option<u32>,
        attempt: u32,
    ) -> Result<JobCompletion, JobError> {
        match AssertUnwindSafe(runner.run(current_job))
            .catch_unwind()
            .await
        {
            Ok(Ok(completion)) => Ok(completion),
            Ok(Err(e)) => {
                let span = Span::current();
                let error = e.to_string();
                span.record("error", true);
                span.record("error.message", tracing::field::display(&error));
                if attempt <= n_warn_attempts.unwrap_or(u32::MAX) {
                    span.record("error.level", tracing::field::display(tracing::Level::WARN));
                } else {
                    span.record(
                        "error.level",
                        tracing::field::display(tracing::Level::ERROR),
                    );
                }
                Err(JobError::JobExecutionError(error))
            }
            Err(panic) => {
                let span = Span::current();
                let message = if let Some(s) = panic.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic payload".to_string()
                };

                span.record("error", true);
                span.record(
                    "error.message",
                    tracing::field::display(&format!("Panic: {message}")),
                );
                span.record(
                    "error.level",
                    tracing::field::display(tracing::Level::ERROR),
                );

                tracing::error!(
                    target: "job.panic",
                    panic_message = %message,
                    panic_backtrace = ?std::backtrace::Backtrace::capture(),
                    "Job panicked during execution"
                );

                Err(JobError::JobExecutionError(format!(
                    "Job panicked: {message}"
                )))
            }
        }
    }

    #[instrument(name = "job.fail_job", skip(self))]
    async fn fail_job(&mut self, id: JobId, error: JobError, attempt: u32) -> Result<(), JobError> {
        self.stop_keep_alive().await;
        let mut op = self.repo.begin_op().await?;
        let mut job = self.repo.find_by_id(id).await?;
        if self.retry_settings.n_attempts.unwrap_or(u32::MAX) > attempt {
            self.rescheduled = true;
            let reschedule_at = self.retry_settings.next_attempt_at(attempt);
            let next_attempt = attempt + 1;
            job.retry_scheduled(error.to_string(), reschedule_at, next_attempt);
            sqlx::query!(
                r#"
                UPDATE job_executions
                SET state = 'pending', execute_at = $2, attempt_index = $3
                WHERE id = $1
              "#,
                id as JobId,
                reschedule_at,
                next_attempt as i32
            )
            .execute(op.as_executor())
            .await?;
        } else {
            job.job_errored(error.to_string());
            sqlx::query!(
                r#"
                DELETE FROM job_executions
                WHERE id = $1
              "#,
                id as JobId
            )
            .execute(op.as_executor())
            .await?;
        }

        self.repo.update_in_op(&mut op, &mut job).await?;

        op.commit().await?;
        Ok(())
    }

    async fn complete_job(
        &mut self,
        mut op: es_entity::DbOp<'_>,
        id: JobId,
    ) -> Result<(), JobError> {
        let mut job = self.repo.find_by_id(&id).await?;
        self.stop_keep_alive().await;
        sqlx::query!(
            r#"
          DELETE FROM job_executions
          WHERE id = $1
        "#,
            id as JobId
        )
        .execute(op.as_executor())
        .await?;
        job.job_completed();
        self.repo.update_in_op(&mut op, &mut job).await?;
        op.commit().await?;
        Ok(())
    }

    async fn reschedule_job(
        &mut self,
        mut op: es_entity::DbOp<'_>,
        id: JobId,
        reschedule_at: DateTime<Utc>,
    ) -> Result<(), JobError> {
        self.rescheduled = true;
        let mut job = self.repo.find_by_id(&id).await?;
        self.stop_keep_alive().await;
        sqlx::query!(
            r#"
          UPDATE job_executions
          SET state = 'pending', execute_at = $2, attempt_index = 1
          WHERE id = $1
        "#,
            id as JobId,
            reschedule_at,
        )
        .execute(op.as_executor())
        .await?;
        job.execution_rescheduled(reschedule_at);
        self.repo.update_in_op(&mut op, &mut job).await?;
        op.commit().await?;
        Ok(())
    }

    async fn stop_keep_alive(&mut self) {
        if let Some(keep_alive) = self.keep_alive.take() {
            keep_alive.stop().await;
        }
    }
}

impl Drop for JobDispatcher {
    fn drop(&mut self) {
        self.tracker.job_completed(self.rescheduled)
    }
}

async fn keep_job_alive(pool: PgPool, id: JobId, job_lost_interval: Duration) {
    let mut failures = 0;
    loop {
        let now = crate::time::now();
        let timeout = match sqlx::query!(
            "UPDATE job_executions SET state = 'running', alive_at = $2 WHERE id = $1",
            id as JobId,
            now,
        )
        .execute(&pool)
        .await
        {
            Ok(_) => {
                failures = 0;
                job_lost_interval / 4
            }
            Err(e) => {
                failures += 1;
                eprintln!("Keep alive job ({id}) errored: {e}");
                Duration::from_millis(50 << failures)
            }
        };
        crate::time::sleep(timeout).await;
    }
}
