use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{entity::*, primitives::*};

use super::dataform_client::{CompilationResult, WorkflowInvocation};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReportEvent {
    Initialized {
        id: ReportId,
        audit_info: AuditInfo,
    },
    CompilationCompleted {
        id: ReportId,
        result: CompilationResult,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    CompilationFailed {
        id: ReportId,
        error: String,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    InvocationCompleted {
        id: ReportId,
        result: WorkflowInvocation,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    InvocationFailed {
        id: ReportId,
        error: String,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
}

impl EntityEvent for ReportEvent {
    type EntityId = ReportId;
    fn event_table_name() -> &'static str {
        "report_events"
    }
}

pub(super) enum ReportGenerationProcessStep {
    Compilation,
    Invocation,
    Upload,
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

impl Report {
    pub(super) fn next_step(&self) -> ReportGenerationProcessStep {
        unimplemented!()
    }

    pub(super) fn compilation_completed(
        &mut self,
        compilation_result: CompilationResult,
        audit_info: AuditInfo,
    ) {
        self.events.push(ReportEvent::CompilationCompleted {
            id: self.id,
            result: compilation_result,
            audit_info,
            recorded_at: Utc::now(),
        });
    }

    pub(super) fn compilation_failed(&mut self, error: String, audit_info: AuditInfo) {
        self.events.push(ReportEvent::CompilationFailed {
            id: self.id,
            error,
            audit_info,
            recorded_at: Utc::now(),
        });
    }

    pub fn compilation_result(&self) -> CompilationResult {
        let res = self.events.iter().rev().find_map(|event| match event {
            ReportEvent::CompilationCompleted { result, .. } => Some(result.clone()),
            _ => None,
        });

        res.expect("Only called after successful compilation")
    }

    pub(super) fn invocation_completed(
        &mut self,
        invocation_result: WorkflowInvocation,
        audit_info: AuditInfo,
    ) {
        self.events.push(ReportEvent::InvocationCompleted {
            id: self.id,
            result: invocation_result,
            audit_info,
            recorded_at: Utc::now(),
        });
    }

    pub(super) fn invocation_failed(&mut self, error: String, audit_info: AuditInfo) {
        self.events.push(ReportEvent::InvocationFailed {
            id: self.id,
            error,
            audit_info,
            recorded_at: Utc::now(),
        });
    }
}

impl TryFrom<EntityEvents<ReportEvent>> for Report {
    type Error = EntityError;

    fn try_from(events: EntityEvents<ReportEvent>) -> Result<Self, Self::Error> {
        let mut builder = ReportBuilder::default();

        for event in events.iter() {
            match event {
                ReportEvent::Initialized { id, .. } => builder = builder.id(*id),
                _ => {}
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
