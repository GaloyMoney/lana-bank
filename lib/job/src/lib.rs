#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod current;
mod entity;
mod executor;
mod handle;
mod new_current;
mod new_executor;
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
use executor::*;
use new_executor::*;
use repo::*;

es_entity::entity_id! { JobId }

#[derive(Clone)]
pub struct Jobs {
    config: JobExecutorConfig,
    repo: JobRepo,
    _executor: JobExecutor,
    registry: Arc<Mutex<Option<JobRegistry>>>,
    executor_handle: Option<Arc<JobExecutorHandle>>,
}

impl Jobs {
    pub fn new(pool: &PgPool, config: JobExecutorConfig) -> Self {
        let repo = JobRepo::new(pool);
        let registry = Arc::new(Mutex::new(Some(JobRegistry::new())));
        let executor = JobExecutor::new(config.clone(), &repo);
        Self {
            repo,
            config,
            _executor: executor,
            registry,
            executor_handle: None,
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
        let mut db = self.repo.begin_op().await?;
        match self.repo.create_in_op(&mut db, new_job).await {
            Err(JobError::DuplicateUniqueJobType) => (),
            Err(e) => return Err(e),
            Ok(job) => {
                self.insert_execution::<<C as JobConfig>::Initializer>(&mut db, &job, None)
                    .await?;
                db.commit().await?;
            }
        }
        Ok(())
    }

    #[instrument(
        name = "jobs.create_and_spawn_in_op",
        skip(self, db, config),
        fields(job_type)
    )]
    pub async fn create_and_spawn_in_op<C: JobConfig>(
        &self,
        db: &mut es_entity::DbOp<'_>,
        id: impl Into<JobId> + std::fmt::Debug,
        config: C,
    ) -> Result<Job, JobError> {
        let job_type = <<C as JobConfig>::Initializer as JobInitializer>::job_type();
        Span::current().record("job_type", tracing::field::display(&job_type));
        let new_job = NewJob::builder()
            .id(id.into())
            .job_type(<<C as JobConfig>::Initializer as JobInitializer>::job_type())
            .config(config)?
            .build()
            .expect("Could not build new job");
        let job = self.repo.create_in_op(db, new_job).await?;
        self.insert_execution::<<C as JobConfig>::Initializer>(db, &job, None)
            .await?;
        Ok(job)
    }

    #[instrument(
        name = "jobs.create_and_spawn_at_in_op",
        skip(self, db, config),
        fields(job_type)
    )]
    pub async fn create_and_spawn_at_in_op<C: JobConfig>(
        &self,
        db: &mut es_entity::DbOp<'_>,
        id: impl Into<JobId> + std::fmt::Debug,
        config: C,
        schedule_at: DateTime<Utc>,
    ) -> Result<Job, JobError> {
        let job_type = <<C as JobConfig>::Initializer as JobInitializer>::job_type();
        Span::current().record("job_type", tracing::field::display(&job_type));
        let new_job = NewJob::builder()
            .id(id.into())
            .job_type(job_type)
            .config(config)?
            .build()
            .expect("Could not build new job");
        let job = self.repo.create_in_op(db, new_job).await?;
        self.insert_execution::<<C as JobConfig>::Initializer>(db, &job, Some(schedule_at))
            .await?;
        Ok(job)
    }

    #[instrument(name = "cala_server.jobs.find", skip(self))]
    pub async fn find(&self, id: JobId) -> Result<Job, JobError> {
        self.repo.find_by_id(id).await
    }

    pub async fn start_executor(&mut self) -> Result<(), JobError> {
        let registry = self
            .registry
            .lock()
            .expect("Couldn't lock Registry Mutex")
            .take()
            .expect("Registry has been consumed by executor");
        self.executor_handle = Some(Arc::new(
            NewJobExecutor::new(self.config.clone(), self.repo.clone(), registry)
                .start()
                .await?,
        ));
        Ok(())
    }

    async fn insert_execution<I: JobInitializer>(
        &self,
        db: &mut es_entity::DbOp<'_>,
        job: &Job,
        schedule_at: Option<DateTime<Utc>>,
    ) -> Result<(), JobError> {
        if job.job_type != I::job_type() {
            return Err(JobError::JobTypeMismatch(
                job.job_type.clone(),
                I::job_type(),
            ));
        }
        sqlx::query!(
            r#"
          INSERT INTO job_executions (id, job_type, reschedule_after, created_at)
          VALUES ($1, $2, $3, $4)
        "#,
            job.id as JobId,
            &job.job_type as &JobType,
            schedule_at.unwrap_or(db.now()),
            db.now()
        )
        .execute(&mut **db.tx())
        .await?;
        Ok(())
    }
}
