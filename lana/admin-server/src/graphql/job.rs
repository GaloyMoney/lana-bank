use async_graphql::*;

use crate::primitives::*;

pub use lana_app::job::{Job as DomainJob, JobStatus};

#[derive(SimpleObject, Clone)]
pub struct Job {
    id: ID,
    job_id: UUID,
    job_type: String,
    status: JobStatus,
    last_error: Option<String>,
    created_at: Timestamp,

    #[graphql(skip)]
    #[allow(dead_code)]
    pub(super) entity: Arc<DomainJob>,
}

impl From<DomainJob> for Job {
    fn from(job: DomainJob) -> Self {
        Job {
            id: job.id.to_global_id(),
            job_id: UUID::from(job.id),
            job_type: job.job_type.to_string(),
            status: job.status(),
            last_error: job.last_error(),
            created_at: job.created_at().into(),
            entity: Arc::new(job),
        }
    }
}
