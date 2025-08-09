#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod current;
mod dispatcher;
mod entity;
mod handle;
mod poller;
mod registry;
mod repo;
mod time;
mod tracker;
mod traits;

pub mod error;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::{Span, instrument};

use std::sync::{Arc, Mutex};

pub use config::*;
pub use current::*;
pub use entity::*;
pub use registry::*;
pub use traits::*;

use error::*;
use poller::*;
use repo::*;

es_entity::entity_id! { JobId }

#[derive(Clone)]
pub struct Jobs {
    config: JobsConfig,
    repo: JobRepo,
    registry: Arc<Mutex<Option<JobRegistry>>>,
    poller_handle: Option<Arc<JobPollerHandle>>,
}

impl Jobs {
    pub fn new(pool: &PgPool, config: JobsConfig) -> Self {
        let repo = JobRepo::new(pool);
        let registry = Arc::new(Mutex::new(Some(JobRegistry::new())));
        Self {
            repo,
            config,
            registry,
            poller_handle: None,
        }
    }

    pub fn add_initializer<I: JobInitializer>(&self, initializer: I) {
        let mut registry = self.registry.lock().expect("Couldn't lock Registry Mutex");
        registry
            .as_mut()
            .expect("Registry has been consumed by executor")
            .add_initializer(initializer);
    }

    pub async fn add_initializer_and_spawn_unique<C: JobConfig>(
        &self,
        initializer: <C as JobConfig>::Initializer,
        config: C,
    ) -> Result<(), JobError> {
        {
            let mut registry = self.registry.lock().expect("Couldn't lock Registry Mutex");
            registry
                .as_mut()
                .expect("Registry has been consumed by executor")
                .add_initializer(initializer);
        }
        let new_job = NewJob::builder()
            .id(JobId::new())
            .unique_per_type(true)
            .job_type(<<C as JobConfig>::Initializer as JobInitializer>::job_type())
            .config(config)?
            .build()
            .expect("Could not build new job");
        let mut op = self.repo.begin_op().await?;
        match self.repo.create_in_op(&mut op, new_job).await {
            Err(JobError::DuplicateUniqueJobType) => (),
            Err(e) => return Err(e),
            Ok(mut job) => {
                let schedule_at = op.now().unwrap_or_else(crate::time::now);
                self.insert_execution::<<C as JobConfig>::Initializer>(
                    &mut op,
                    &mut job,
                    schedule_at,
                )
                .await?;
                op.commit().await?;
            }
        }
        Ok(())
    }

    #[instrument(
        name = "job.create_and_spawn_in_op",
        skip(self, op, config),
        fields(job_type, now)
    )]
    pub async fn create_and_spawn_in_op<C: JobConfig>(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        job_id: impl Into<JobId> + std::fmt::Debug,
        config: C,
    ) -> Result<Job, JobError> {
        let job_type = <<C as JobConfig>::Initializer as JobInitializer>::job_type();
        Span::current().record("job_type", tracing::field::display(&job_type));
        let new_job = NewJob::builder()
            .id(job_id.into())
            .job_type(<<C as JobConfig>::Initializer as JobInitializer>::job_type())
            .config(config)?
            .build()
            .expect("Could not build new job");
        let mut job = self.repo.create_in_op(op, new_job).await?;
        let schedule_at = op.now().unwrap_or_else(crate::time::now);
        self.insert_execution::<<C as JobConfig>::Initializer>(op, &mut job, schedule_at)
            .await?;
        Ok(job)
    }

    #[instrument(
        name = "job.create_and_spawn_at_in_op",
        skip(self, op, config),
        fields(job_type, now)
    )]
    pub async fn create_and_spawn_at_in_op<C: JobConfig>(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        job_id: impl Into<JobId> + std::fmt::Debug,
        config: C,
        schedule_at: DateTime<Utc>,
    ) -> Result<Job, JobError> {
        let job_type = <<C as JobConfig>::Initializer as JobInitializer>::job_type();
        Span::current().record("job_type", tracing::field::display(&job_type));
        let new_job = NewJob::builder()
            .id(job_id.into())
            .job_type(job_type)
            .config(config)?
            .build()
            .expect("Could not build new job");
        let mut job = self.repo.create_in_op(op, new_job).await?;
        self.insert_execution::<<C as JobConfig>::Initializer>(op, &mut job, schedule_at)
            .await?;
        Ok(job)
    }

    #[instrument(name = "job.find", skip(self))]
    pub async fn find(&self, id: JobId) -> Result<Job, JobError> {
        self.repo.find_by_id(id).await
    }

    pub async fn start_poll(&mut self) -> Result<(), JobError> {
        let registry = self
            .registry
            .lock()
            .expect("Couldn't lock Registry Mutex")
            .take()
            .expect("Registry has been consumed by executor");
        self.poller_handle = Some(Arc::new(
            JobPoller::new(self.config.clone(), self.repo.clone(), registry)
                .start()
                .await?,
        ));
        Ok(())
    }

    async fn insert_execution<I: JobInitializer>(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        job: &mut Job,
        schedule_at: DateTime<Utc>,
    ) -> Result<(), JobError> {
        if job.job_type != I::job_type() {
            return Err(JobError::JobTypeMismatch(
                job.job_type.clone(),
                I::job_type(),
            ));
        }
        sqlx::query!(
            r#"
          INSERT INTO job_executions (id, job_type, execute_at, alive_at, created_at)
          VALUES ($1, $2, $3, COALESCE($4, NOW()), COALESCE($4, NOW()))
        "#,
            job.id as JobId,
            &job.job_type as &JobType,
            schedule_at,
            op.now()
        )
        .execute(op.as_executor())
        .await?;
        job.execution_scheduled(schedule_at);
        self.repo.update_in_op(op, job).await?;
        Ok(())
    }
}
