use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ReportFile {
    pub extension: String,
    pub path_in_bucket: String,
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ReportId")]
pub enum ReportEvent {
    Initialized {
        id: ReportId,
        external_id: String,
        run_id: ReportRunId,
        name: String,
        norm: String,
        files: Vec<ReportFile>,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Report {
    pub id: ReportId,
    pub run_id: ReportRunId,
    pub external_id: String,
    pub name: String,
    pub norm: String,
    pub files: Vec<ReportFile>,
    events: EntityEvents<ReportEvent>,
}

impl Report {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for report")
    }
}

impl TryFromEvents<ReportEvent> for Report {
    fn try_from_events(events: EntityEvents<ReportEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ReportBuilder::default();

        for event in events.iter_all() {
            match event {
                ReportEvent::Initialized {
                    id,
                    external_id,
                    run_id,
                    name,
                    norm,
                    files,
                } => {
                    builder = builder
                        .id(*id)
                        .external_id(external_id.clone())
                        .run_id(*run_id)
                        .name(name.clone())
                        .norm(norm.clone())
                        .files(files.clone())
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
    pub(super) external_id: String,
    #[builder(setter(into))]
    pub(super) run_id: ReportRunId,
    #[builder(setter(into))]
    pub(super) name: String,
    #[builder(setter(into))]
    pub(super) norm: String,
    pub(super) files: Vec<ReportFile>,
}

impl NewReport {
    pub fn builder() -> NewReportBuilder {
        let report_id = ReportId::new();

        let mut builder = NewReportBuilder::default();
        builder.id(report_id);
        builder
    }
}

impl IntoEvents<ReportEvent> for NewReport {
    fn into_events(self) -> EntityEvents<ReportEvent> {
        EntityEvents::init(
            self.id,
            [ReportEvent::Initialized {
                id: self.id,
                external_id: self.external_id,
                run_id: self.run_id,
                name: self.name,
                norm: self.norm,
                files: self.files,
            }],
        )
    }
}
