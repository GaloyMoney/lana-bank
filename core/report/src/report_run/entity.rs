use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ReportRunState {
    Queued,
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ReportRunType {
    Scheduled,
    Manual,
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ReportRunId")]
pub enum ReportRunEvent {
    Initialized {
        id: ReportRunId,
        external_id: String,
    },
    StateUpdated {
        execution_date: DateTime<Utc>,
        state: ReportRunState,
        run_type: ReportRunType,
        start_date: DateTime<Utc>,
        end_date: Option<DateTime<Utc>>,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct ReportRun {
    pub id: ReportRunId,
    pub external_id: String,
    #[builder(setter(strip_option), default)]
    pub execution_date: Option<DateTime<Utc>>,
    #[builder(setter(strip_option), default)]
    pub state: Option<ReportRunState>,
    #[builder(setter(strip_option), default)]
    pub run_type: Option<ReportRunType>,
    #[builder(setter(strip_option), default)]
    pub start_date: Option<DateTime<Utc>>,
    #[builder(setter(strip_option), default)]
    pub end_date: Option<DateTime<Utc>>,
    events: EntityEvents<ReportRunEvent>,
}

impl TryFromEvents<ReportRunEvent> for ReportRun {
    fn try_from_events(events: EntityEvents<ReportRunEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ReportRunBuilder::default();

        for event in events.iter_all() {
            match event {
                ReportRunEvent::Initialized { id, external_id } => {
                    builder = builder.id(*id).external_id(external_id.clone())
                }
                ReportRunEvent::StateUpdated {
                    execution_date,
                    state,
                    run_type,
                    start_date,
                    end_date,
                } => {
                    builder = builder
                        .execution_date(*execution_date)
                        .state(*state)
                        .run_type(*run_type)
                        .start_date(*start_date);
                    if let Some(end_date_val) = end_date {
                        builder = builder.end_date(*end_date_val);
                    }
                }
            }
        }

        builder.events(events).build()
    }
}

impl ReportRun {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for report run")
    }

    pub fn update_state(
        &mut self,
        state: ReportRunState,
        run_type: ReportRunType,
        execution_date: DateTime<Utc>,
        start_date: DateTime<Utc>,
        end_date: Option<DateTime<Utc>>,
    ) {
        self.state = Some(state);
        self.run_type = Some(run_type);
        self.execution_date = Some(execution_date);
        self.start_date = Some(start_date);
        self.end_date = end_date;

        self.events.push(ReportRunEvent::StateUpdated {
            state,
            run_type,
            execution_date,
            start_date,
            end_date,
        });
    }
}

#[derive(Debug, Builder)]
pub struct NewReportRun {
    #[builder(setter(into))]
    pub(super) id: ReportRunId,
    #[builder(setter(into))]
    pub(super) external_id: String,
}

impl NewReportRun {
    pub fn builder() -> NewReportRunBuilder {
        let report_run_id = ReportRunId::new();

        let mut builder = NewReportRunBuilder::default();
        builder.id(report_run_id);
        builder
    }
}

impl IntoEvents<ReportRunEvent> for NewReportRun {
    fn into_events(self) -> EntityEvents<ReportRunEvent> {
        EntityEvents::init(
            self.id,
            [ReportRunEvent::Initialized {
                id: self.id,
                external_id: self.external_id,
            }],
        )
    }
}
