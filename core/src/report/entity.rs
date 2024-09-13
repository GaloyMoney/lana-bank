use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{entity::*, primitives::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReportEvent {
    Initialized { id: ReportId, audit_info: AuditInfo },
}

impl EntityEvent for ReportEvent {
    type EntityId = ReportId;
    fn event_table_name() -> &'static str {
        "report_events"
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EntityError"))]
pub struct Report {
    pub id: ReportId,
    pub(super) events: EntityEvents<ReportEvent>,
}

impl Entity for Report {
    type Event = ReportEvent;
}

impl TryFrom<EntityEvents<ReportEvent>> for Report {
    type Error = EntityError;

    fn try_from(events: EntityEvents<ReportEvent>) -> Result<Self, Self::Error> {
        let mut builder = ReportBuilder::default();

        for event in events.iter() {
            match event {
                ReportEvent::Initialized { id, .. } => builder = builder.id(*id),
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
    pub(super) audit_info: AuditInfo,
}

impl NewReport {
    pub fn builder() -> NewReportBuilder {
        NewReportBuilder::default()
    }

    pub(super) fn initial_events(self) -> EntityEvents<ReportEvent> {
        EntityEvents::init(
            self.id,
            [ReportEvent::Initialized {
                id: self.id,
                audit_info: self.audit_info,
            }],
        )
    }
}
