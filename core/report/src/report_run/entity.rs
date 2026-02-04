use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ReportRunState {
    Queued,
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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
        state: ReportRunState,
        run_type: ReportRunType,
        start_time: Option<DateTime<Utc>>,
    },
    StateUpdated {
        state: ReportRunState,
        run_type: ReportRunType,
        start_time: Option<DateTime<Utc>>,
    },
}

#[derive(EsEntity, Builder, Clone)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct ReportRun {
    pub id: ReportRunId,
    pub external_id: String,
    pub state: ReportRunState,
    pub run_type: ReportRunType,
    #[builder(default)]
    pub start_time: Option<DateTime<Utc>>,
    events: EntityEvents<ReportRunEvent>,
}

impl TryFromEvents<ReportRunEvent> for ReportRun {
    fn try_from_events(events: EntityEvents<ReportRunEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ReportRunBuilder::default();

        for event in events.iter_all() {
            match event {
                ReportRunEvent::Initialized {
                    id,
                    external_id,
                    state,
                    run_type,
                    start_time,
                } => {
                    builder = builder
                        .id(*id)
                        .external_id(external_id.clone())
                        .state(*state)
                        .run_type(*run_type)
                        .start_time(*start_time)
                }
                ReportRunEvent::StateUpdated {
                    state,
                    run_type,
                    start_time,
                } => {
                    builder = builder
                        .state(*state)
                        .run_type(*run_type)
                        .start_time(*start_time)
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
        start_time: Option<DateTime<Utc>>,
    ) -> Idempotent<()> {
        if self.state == state && self.run_type == run_type && self.start_time == start_time {
            return Idempotent::AlreadyApplied;
        }

        self.state = state;
        self.run_type = run_type;
        self.start_time = start_time;

        self.events.push(ReportRunEvent::StateUpdated {
            state,
            run_type,
            start_time,
        });

        Idempotent::Executed(())
    }
}

#[derive(Debug, Builder)]
pub struct NewReportRun {
    #[builder(setter(into))]
    pub(super) id: ReportRunId,
    #[builder(setter(into))]
    pub(super) external_id: String,
    #[builder(setter(into))]
    pub(super) state: ReportRunState,
    #[builder(setter(into))]
    pub(super) run_type: ReportRunType,
    #[builder(default)]
    pub(super) start_time: Option<DateTime<Utc>>,
}

impl NewReportRun {
    pub fn builder() -> NewReportRunBuilder {
        let report_run_id = ReportRunId::new();

        let mut builder = NewReportRunBuilder::default();
        builder.id(report_run_id);
        builder.state(ReportRunState::Queued);
        builder.run_type(ReportRunType::Manual);
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
                state: self.state,
                run_type: self.run_type,
                start_time: self.start_time,
            }],
        )
    }
}
