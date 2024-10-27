use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use std::borrow::Cow;

use es_entity::*;

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
use crate::error::JobError;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobId {
    Id(uuid::Uuid),
    Unique(JobType),
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
    pub(super) events: EntityEvents<JobEvent>,
}

impl Job {
    pub fn config<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.config.clone())
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
                        .id(id.clone())
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
    pub(super) id: JobId,
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
            self.id.clone(),
            [JobEvent::Initialized {
                id: self.id,
                job_type: self.job_type,
                config: self.config,
            }],
        )
    }
}

mod id_sqlx {
    use sqlx::{encode::*, postgres::*, *};

    use std::{fmt, str::FromStr};

    use super::JobId;
    use crate::JobType;

    impl fmt::Display for JobId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                JobId::Id(uuid) => write!(f, "id:{}", uuid),
                JobId::Unique(job_type) => write!(f, "unique:{:?}", job_type),
            }
        }
    }

    impl FromStr for JobId {
        type Err = Box<dyn std::error::Error + Sync + Send>;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.split_once(':') {
                Some(("id", uuid_str)) => Ok(JobId::Id(uuid::Uuid::parse_str(uuid_str)?)),
                Some(("unique", job_type_str)) => Ok(JobId::Unique(JobType::from_string(
                    job_type_str.to_string(),
                ))),
                _ => Err("Invalid format".into()),
            }
        }
    }
    impl Type<Postgres> for JobId {
        fn type_info() -> PgTypeInfo {
            <String as Type<Postgres>>::type_info()
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            <String as Type<Postgres>>::compatible(ty)
        }
    }

    impl<'q> sqlx::Encode<'q, Postgres> for JobId {
        fn encode_by_ref(
            &self,
            buf: &mut sqlx::postgres::PgArgumentBuffer,
        ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
            <String as sqlx::Encode<'_, Postgres>>::encode(self.to_string(), buf)
        }
    }

    impl<'r> sqlx::Decode<'r, Postgres> for JobId {
        fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let s = <String as sqlx::Decode<Postgres>>::decode(value)?;
            s.parse()
        }
    }

    impl PgHasArrayType for JobId {
        fn array_type_info() -> sqlx::postgres::PgTypeInfo {
            <String as sqlx::postgres::PgHasArrayType>::array_type_info()
        }
    }
}
