use async_trait::async_trait;

use super::{
    current::CurrentJob,
    entity::{Job, JobType},
};

pub trait JobInitializer: Send + Sync + 'static {
    fn job_type() -> JobType
    where
        Self: Sized;

    fn init(&self, job: &Job) -> Result<Box<dyn JobRunner>, Box<dyn std::error::Error>>;
}

pub enum JobCompletion {
    Complete,
    CompleteWithTx(sqlx::Transaction<'static, sqlx::Postgres>),
    Pause,
    PauseWithTx(sqlx::Transaction<'static, sqlx::Postgres>),
}

#[async_trait]
pub trait JobRunner: Send + Sync + 'static {
    async fn run(
        &self,
        current_job: CurrentJob,
    ) -> Result<JobCompletion, Box<dyn std::error::Error>>;
}
