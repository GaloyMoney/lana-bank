#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod current;
mod entity;
mod executor;
mod registry;
mod repo;
mod traits;

pub mod error;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tokio::sync::RwLock;
use tracing::instrument;

use std::sync::Arc;

pub use config::*;
pub use current::*;
pub use entity::*;
pub use registry::*;
pub use traits::*;

use error::*;
use executor::*;
use repo::*;

#[derive(Clone)]
pub struct Jobs {
    _pool: PgPool,
    repo: JobRepo,
    executor: JobExecutor,
    registry: Arc<RwLock<JobRegistry>>,
}

impl Jobs {
    pub fn new(pool: &PgPool, config: JobExecutorConfig) -> Self {
        let repo = JobRepo::new(pool);
        let registry = Arc::new(RwLock::new(JobRegistry::new()));
        let executor = JobExecutor::new(pool, config, Arc::clone(&registry), &repo);
        Self {
            _pool: pool.clone(),
            repo,
            executor,
            registry,
        }
    }

    pub fn add_initializer<I: JobInitializer>(&self, initializer: I) {
        let mut registry = self.registry.try_write().expect("Could not lock registry");
        registry.add_initializer(initializer);
    }

    #[instrument(name = "lava.jobs.create_and_spawn", skip(self, db, config))]
    pub async fn create_and_spawn_in_tx<I: JobInitializer, C: serde::Serialize>(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: impl Into<JobId> + std::fmt::Debug,
        config: C,
    ) -> Result<Job, JobError> {
        let new_job = NewJob::builder()
            .id(id.into())
            .job_type(<I as JobInitializer>::job_type())
            .config(config)?
            .build()
            .expect("Could not build new job");
        let job = self.repo.create_in_tx(db, new_job).await?;
        self.executor.spawn_job::<I>(db, &job, None).await?;
        Ok(job)
    }

    #[instrument(name = "lava.jobs.create_and_spawn_at", skip(self, db, config))]
    pub async fn create_and_spawn_at_in_tx<I: JobInitializer, C: serde::Serialize>(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: impl Into<JobId> + std::fmt::Debug,
        config: C,
        schedule_at: DateTime<Utc>,
    ) -> Result<Job, JobError> {
        let new_job = NewJob::builder()
            .id(id.into())
            .job_type(<I as JobInitializer>::job_type())
            .config(config)?
            .build()
            .expect("Could not build new job");
        let job = self.repo.create_in_tx(db, new_job).await?;
        self.executor
            .spawn_job::<I>(db, &job, Some(schedule_at))
            .await?;
        Ok(job)
    }

    #[instrument(name = "cala_server.jobs.find", skip(self))]
    pub async fn find(&self, id: JobId) -> Result<Job, JobError> {
        self.repo.find_by_id(id).await
    }

    pub async fn start_poll(&mut self) -> Result<(), JobError> {
        self.executor.start_poll().await
    }
}
