use sqlx::PgPool;

use es_entity::*;

use super::{entity::*, error::*};
use crate::JobId;

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Job",
    err = "JobError",
    columns(
        job_type(ty = "JobType", update(persist = false)),
        unique_per_type(ty = "bool", update(persist = false)),
    )
)]
pub struct JobRepo {
    pool: PgPool,
}

impl JobRepo {
    pub(super) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub async fn init_pool() -> anyhow::Result<sqlx::PgPool> {
        let pg_host = std::env::var("PGHOST").unwrap_or("localhost".to_string());
        let pg_con = format!("postgres://user:password@{pg_host}:5433/pg");
        let pool = sqlx::PgPool::connect(&pg_con).await?;
        Ok(pool)
    }

    #[tokio::test]
    async fn unique_per_job_type() -> anyhow::Result<()> {
        let pool = init_pool().await?;
        let repo = JobRepo::new(&pool);
        let type_a = JobType::from_owned(uuid::Uuid::new_v4().to_string());
        let type_b = JobType::from_owned(uuid::Uuid::new_v4().to_string());
        let type_c = JobType::from_owned(uuid::Uuid::new_v4().to_string());

        let a_id = JobId::new();
        let new_job = NewJob::builder()
            .id(a_id)
            .unique_per_type(true)
            .job_type(type_a.clone())
            .config(serde_json::json!({}))?
            .build()
            .expect("Could not build new job");
        repo.create(new_job).await?;

        // Different id same type
        let new_job = NewJob::builder()
            .id(JobId::new())
            .unique_per_type(true)
            .job_type(type_a.clone())
            .config(serde_json::json!({}))?
            .build()
            .expect("Could not build new job");
        let res = repo.create(new_job).await;
        assert!(matches!(res, Err(JobError::DuplicateUniqueJobType)));

        // Same type same id
        let new_job = NewJob::builder()
            .id(a_id)
            .unique_per_type(true)
            .job_type(type_a.clone())
            .config(serde_json::json!({}))?
            .build()
            .expect("Could not build new job");
        let res = repo.create(new_job).await;
        assert!(matches!(res, Err(JobError::DuplicateId)));

        let new_job = NewJob::builder()
            .id(JobId::new())
            .unique_per_type(true)
            .job_type(type_b)
            .config(serde_json::json!({}))?
            .build()
            .expect("Could not build new job");
        repo.create(new_job).await?;

        let new_job = NewJob::builder()
            .id(JobId::new())
            .job_type(type_c.clone())
            .config(serde_json::json!({}))?
            .build()
            .expect("Could not build new job");
        repo.create(new_job).await?;
        let new_job = NewJob::builder()
            .id(JobId::new())
            .job_type(type_c.clone())
            .config(serde_json::json!({}))?
            .build()
            .expect("Could not build new job");
        repo.create(new_job).await?;
        let new_job = NewJob::builder()
            .id(a_id)
            .job_type(type_c)
            .config(serde_json::json!({}))?
            .build()
            .expect("Could not build new job");
        let res = repo.create(new_job).await;
        assert!(matches!(res, Err(JobError::DuplicateId)));

        Ok(())
    }
}
