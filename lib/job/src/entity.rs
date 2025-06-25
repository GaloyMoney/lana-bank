#[cfg(feature = "graphql")]
use async_graphql::*;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::borrow::Cow;

use es_entity::*;

use crate::{JobId, error::JobError};

#[derive(Clone, Eq, Hash, PartialEq, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct JobType(Cow<'static, str>);
impl JobType {
    pub const fn new(job_type: &'static str) -> Self {
        JobType(Cow::Borrowed(job_type))
    }

    #[cfg(test)]
    pub(crate) fn from_owned(job_type: String) -> Self {
        JobType(Cow::Owned(job_type))
    }
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum, Copy, Eq, PartialEq))]
pub enum JobStatus {
    Running,
    Completed,
    Errored,
}

#[derive(EsEvent, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "JobId")]
pub enum JobEvent {
    Initialized {
        id: JobId,
        job_type: JobType,
        config: serde_json::Value,
    },
    Errored {
        error: String,
    },
    Completed,
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Job {
    pub id: JobId,
    pub job_type: JobType,
    config: serde_json::Value,
    events: EntityEvents<JobEvent>,
}

impl Job {
    pub fn config<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.config.clone())
    }

    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn status(&self) -> JobStatus {
        for event in self.events.iter_all().rev() {
            match event {
                JobEvent::Completed => return JobStatus::Completed,
                JobEvent::Errored { .. } => return JobStatus::Errored,
                _ => {}
            }
        }
        JobStatus::Running
    }

    pub fn last_error(&self) -> Option<String> {
        // If job is completed, return None even if there were previous errors
        if matches!(self.status(), JobStatus::Completed) {
            return None;
        }

        for event in self.events.iter_all().rev() {
            if let JobEvent::Errored { error } = event {
                return Some(error.clone());
            }
        }
        None
    }

    pub(super) fn completed(&mut self) {
        self.events.push(JobEvent::Completed);
    }

    pub(super) fn fail(&mut self, error: String) {
        self.events.push(JobEvent::Errored { error });
    }
}

impl TryFromEvents<JobEvent> for Job {
    fn try_from_events(events: EntityEvents<JobEvent>) -> Result<Self, EsEntityError> {
        let mut builder = JobBuilder::default();
        for event in events.iter_all() {
            match event {
                JobEvent::Initialized {
                    id,
                    job_type,
                    config,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .job_type(job_type.clone())
                        .config(config.clone())
                }
                JobEvent::Errored { .. } => {}
                JobEvent::Completed => {}
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewJob {
    #[builder(setter(into))]
    pub(super) id: JobId,
    #[builder(default)]
    pub(super) unique_per_type: bool,
    pub(super) job_type: JobType,
    #[builder(setter(custom))]
    pub(super) config: serde_json::Value,
}

impl NewJob {
    pub fn builder() -> NewJobBuilder {
        NewJobBuilder::default()
    }
}

impl NewJobBuilder {
    pub fn config<C: serde::Serialize>(&mut self, config: C) -> Result<&mut Self, JobError> {
        self.config =
            Some(serde_json::to_value(config).map_err(JobError::CouldNotSerializeConfig)?);
        Ok(self)
    }
}

impl IntoEvents<JobEvent> for NewJob {
    fn into_events(self) -> EntityEvents<JobEvent> {
        EntityEvents::init(
            self.id,
            [JobEvent::Initialized {
                id: self.id,
                job_type: self.job_type,
                config: self.config,
            }],
        )
    }
}
