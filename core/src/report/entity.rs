use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{entity::*, primitives::*};

use super::dataform_client::{CompilationResult, UploadResult, WorkflowInvocation};

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
    UploadCompleted {
        id: ReportId,
        gcs_path: String,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    UploadFailed {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        let last_step = self.events.iter().rev().find_map(|event| match event {
            ReportEvent::CompilationCompleted { .. } | ReportEvent::InvocationFailed { .. } => {
                Some(ReportGenerationProcessStep::Invocation)
            }
            ReportEvent::InvocationCompleted { .. } | ReportEvent::UploadFailed { .. } => {
                Some(ReportGenerationProcessStep::Upload)
            }

            _ => None,
        });

        last_step.unwrap_or(ReportGenerationProcessStep::Compilation)
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

    pub(super) fn upload_completed(&mut self, upload_result: UploadResult, audit_info: AuditInfo) {
        self.events.push(ReportEvent::UploadCompleted {
            id: self.id,
            gcs_path: upload_result.gcs_path,
            audit_info,
            recorded_at: Utc::now(),
        });
    }

    pub(super) fn upload_failed(&mut self, error: String, audit_info: AuditInfo) {
        self.events.push(ReportEvent::UploadFailed {
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

#[cfg(test)]
mod test {
    use super::*;

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: Subject::from(UserId::new()),
        }
    }

    fn init_report(events: Vec<ReportEvent>) -> Report {
        Report::try_from(EntityEvents::init(ReportId::new(), events)).unwrap()
    }

    #[test]
    fn next_step() {
        let id = ReportId::new();
        let mut events = vec![ReportEvent::Initialized {
            id,
            audit_info: dummy_audit_info(),
        }];
        assert_eq!(
            init_report(events.clone()).next_step(),
            ReportGenerationProcessStep::Compilation
        );

        events.push(ReportEvent::CompilationFailed {
            id,
            error: "".to_string(),
            audit_info: dummy_audit_info(),
            recorded_at: Utc::now(),
        });
        assert_eq!(
            init_report(events.clone()).next_step(),
            ReportGenerationProcessStep::Compilation
        );

        events.push(ReportEvent::CompilationCompleted {
            id,
            result: CompilationResult::default(),
            audit_info: dummy_audit_info(),
            recorded_at: Utc::now(),
        });
        assert_eq!(
            init_report(events.clone()).next_step(),
            ReportGenerationProcessStep::Invocation
        );

        events.push(ReportEvent::InvocationFailed {
            id,
            error: "".to_string(),
            audit_info: dummy_audit_info(),
            recorded_at: Utc::now(),
        });
        assert_eq!(
            init_report(events.clone()).next_step(),
            ReportGenerationProcessStep::Invocation
        );

        events.push(ReportEvent::InvocationCompleted {
            id,
            result: WorkflowInvocation {
                name: "".to_string(),
                state: crate::report::dataform_client::WorkflowInvocationState::Succeeded,
            },
            audit_info: dummy_audit_info(),
            recorded_at: Utc::now(),
        });
        assert_eq!(
            init_report(events.clone()).next_step(),
            ReportGenerationProcessStep::Upload
        );

        events.push(ReportEvent::UploadFailed {
            id,
            error: "".to_string(),
            audit_info: dummy_audit_info(),
            recorded_at: Utc::now(),
        });
        assert_eq!(
            init_report(events.clone()).next_step(),
            ReportGenerationProcessStep::Upload
        );
    }
}
