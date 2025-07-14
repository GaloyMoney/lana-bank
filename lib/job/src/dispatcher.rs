use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use tracing::{Span, instrument};

use std::sync::Arc;

use super::{
    JobId, error::JobError, new_current::NewCurrentJob, repo::JobRepo, tracker::JobTracker,
    traits::*,
};

#[derive(Debug)]
pub struct PolledJob {
    pub id: JobId,
    pub job_type: String,
    pub data_json: Option<JsonValue>,
    pub attempt: u32,
}

pub(crate) struct JobDispatcher {
    repo: JobRepo,
    retry_settings: RetrySettings,
    runner: Option<Box<dyn JobRunner>>,
    tracker: Arc<JobTracker>,
}
impl JobDispatcher {
    pub fn new(
        repo: JobRepo,
        tracker: Arc<JobTracker>,
        retry_settings: RetrySettings,
        runner: Box<dyn JobRunner>,
    ) -> Self {
        Self {
            repo,
            retry_settings,
            runner: Some(runner),
            tracker,
        }
    }

    #[instrument(name = "job.start_job", skip_all,
        fields(job_id, job_type, attempt, error, error.level, error.message, conclusion),
    err)]
    pub async fn start_job(mut self, polled_job: PolledJob) -> Result<(), JobError> {
        let job = self.repo.find_by_id(polled_job.id).await?;
        let span = Span::current();
        span.record("job_id", tracing::field::display(job.id));
        span.record("job_type", tracing::field::display(&job.job_type));
        span.record("attempt", polled_job.attempt);
        let current_job = NewCurrentJob::new(
            self.repo.pool().clone(),
            polled_job.id,
            polled_job.attempt,
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
                let t = op.now();
                Self::reschedule_job(op, job.id, t).await?;
            }
            Ok(JobCompletion::RescheduleNowWithOp(op)) => {
                span.record("conclusion", "RescheduleNowWithOp");
                let t = op.now();
                Self::reschedule_job(op, job.id, t).await?;
            }
            Ok(JobCompletion::RescheduleIn(d)) => {
                span.record("conclusion", "RescheduleIn");
                let op = self.repo.begin_op().await?;
                let t = op.now() + d;
                Self::reschedule_job(op, job.id, t).await?;
            }
            Ok(JobCompletion::RescheduleInWithOp(d, op)) => {
                span.record("conclusion", "RescheduleInWithOp");
                let t = op.now() + d;
                Self::reschedule_job(op, job.id, t).await?;
            }
            Ok(JobCompletion::RescheduleAt(t)) => {
                span.record("conclusion", "RescheduleAt");
                let op = self.repo.begin_op().await?;
                Self::reschedule_job(op, job.id, t).await?;
            }
            Ok(JobCompletion::RescheduleAtWithOp(op, t)) => {
                span.record("conclusion", "RescheduleAtWithOp");
                Self::reschedule_job(op, job.id, t).await?;
            }
        }
        Ok(())
    }

    async fn dispatch_job(
        runner: Box<dyn JobRunner>,
        current_job: NewCurrentJob,
        n_warn_attempts: Option<u32>,
        attempt: u32,
    ) -> Result<JobCompletion, JobError> {
        runner.new_run(current_job).await.map_err(|e| {
            let error = e.to_string();
            Span::current().record("error", true);
            Span::current().record("error.message", tracing::field::display(&error));
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
        })
    }

    #[instrument(name = "job.fail_job", skip(self))]
    async fn fail_job(&self, id: JobId, error: JobError, attempt: u32) -> Result<(), JobError> {
        let mut job = self.repo.find_by_id(id).await?;
        job.fail(error.to_string());
        let mut op = self.repo.begin_op().await?;
        self.repo.update_in_op(&mut op, &mut job).await?;
        if self.retry_settings.n_attempts.unwrap_or(u32::MAX) > attempt {
            let reschedule_at = self.retry_settings.next_attempt_at(attempt);
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

    async fn complete_job(&self, mut op: es_entity::DbOp<'_>, id: JobId) -> Result<(), JobError> {
        let mut job = self.repo.find_by_id(&id).await?;
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
        self.repo.update_in_op(&mut op, &mut job).await?;
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
}

impl Drop for JobDispatcher {
    fn drop(&mut self) {
        self.tracker.job_completed()
    }
}
