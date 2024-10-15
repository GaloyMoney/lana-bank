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

#[derive(
    sqlx::Type,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct JobId(uuid::Uuid);
impl JobId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
impl From<uuid::Uuid> for JobId {
    fn from(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }
}
impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

    #[instrument(name = "lava.jobs.create_and_spawn", skip(self, db, initial_data))]
    pub async fn create_and_spawn_in_tx<I: JobInitializer, D: serde::Serialize>(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: impl Into<JobId> + std::fmt::Debug,
        name: String,
        initial_data: D,
    ) -> Result<Job, JobError> {
        let new_job = Job::new(
            name,
            Some(id.into()),
            <I as JobInitializer>::job_type(),
            initial_data,
        );
        let job = self.repo.create_in_tx(db, new_job).await?;
        self.executor.spawn_job::<I>(db, &job, None).await?;
        Ok(job)
    }

    #[instrument(name = "lava.jobs.create_and_spawn_at", skip(self, db, initial_data))]
    pub async fn create_and_spawn_at_in_tx<I: JobInitializer, D: serde::Serialize>(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        name: String,
        initial_data: D,
        schedule_at: DateTime<Utc>,
    ) -> Result<Job, JobError> {
        let new_job = Job::new(name, None, <I as JobInitializer>::job_type(), initial_data);
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
