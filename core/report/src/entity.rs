use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ReportId")]
pub enum ReportEvent {
    Initialized {
        id: ReportId,
        date: DateTime<Utc>,
        path_in_bucket: String,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Report {
    pub id: ReportId,
    pub date: DateTime<Utc>,
    pub path_in_bucket: String,
    events: EntityEvents<ReportEvent>,
}

impl core::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Report({}): {}/{}",
            self.id, self.date, self.path_in_bucket
        )
    }
}

impl TryFromEvents<ReportEvent> for Report {
    fn try_from_events(events: EntityEvents<ReportEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ReportBuilder::default();

        for event in events.iter_all() {
            match event {
                ReportEvent::Initialized {
                    id,
                    date,
                    path_in_bucket,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .date(*date)
                        .path_in_bucket(path_in_bucket.clone());
                }
            }
        }

        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewReport {
    #[builder(setter(into))]
    pub(super) id: ReportId,
    #[builder(setter(into))]
    pub(super) date: DateTime<Utc>,
    #[builder(setter(into))]
    pub(super) path_in_bucket: String,
    pub(super) audit_info: AuditInfo,
}

impl NewReport {
    pub fn builder() -> NewReportBuilder {
        NewReportBuilder::default()
    }
}

impl IntoEvents<ReportEvent> for NewReport {
    fn into_events(self) -> EntityEvents<ReportEvent> {
        EntityEvents::init(
            self.id,
            [ReportEvent::Initialized {
                id: self.id,
                date: self.date,
                path_in_bucket: self.path_in_bucket,
                audit_info: self.audit_info,
            }],
        )
    }
}
