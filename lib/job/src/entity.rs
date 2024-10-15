use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use std::borrow::Cow;

pub use super::JobId;

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct JobType(Cow<'static, str>);
impl JobType {
    pub const fn new(job_type: &'static str) -> Self {
        JobType(Cow::Borrowed(job_type))
    }

    pub(super) fn from_string(job_type: String) -> Self {
        JobType(Cow::Owned(job_type))
    }
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct Job {
    pub id: JobId,
    pub name: String,
    pub job_type: JobType,
    pub last_error: Option<String>,
    data: serde_json::Value,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Job {
    pub(super) fn new<T: serde::Serialize>(
        name: String,
        id: Option<JobId>,
        job_type: JobType,
        initial_data: T,
    ) -> Self {
        Self {
            id: id.unwrap_or(JobId::new()),
            name,
            job_type,
            data: serde_json::to_value(initial_data).expect("could not serialize job data"),
            last_error: None,
            completed_at: None,
        }
    }

    pub fn data<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.data.clone())
    }

    pub(super) fn success(&mut self) {
        self.completed_at = Some(Utc::now());
        self.last_error = None;
    }

    pub(super) fn fail(&mut self, error: String) {
        self.last_error = Some(error);
    }
}
