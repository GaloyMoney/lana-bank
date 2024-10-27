use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::{audit::AuditInfo, entity::*, primitives::*, storage::LocationInCloud};

use super::{
    dataform_client::{CompilationResult, WorkflowInvocation},
    upload::ReportFileUpload,
};

#[derive(Debug)]
pub struct ReportLocationInCloud {
    pub report_name: String,
    pub bucket: String,
    pub path_in_bucket: String,
}

impl<'a> From<&'a ReportLocationInCloud> for LocationInCloud<'a> {
    fn from(meta: &'a ReportLocationInCloud) -> Self {
        LocationInCloud {
            bucket: &meta.bucket,
            path_in_bucket: &meta.path_in_bucket,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReportDownloadLink {
    pub report_name: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct GeneratedReportDownloadLinks {
    pub report_id: ReportId,
    pub links: Vec<ReportDownloadLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReportEvent {
    Initialized {
        id: ReportId,
        audit_info: AuditInfo,
    },
    CompilationCompleted {
        result: CompilationResult,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    CompilationFailed {
        error: String,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    InvocationCompleted {
        result: WorkflowInvocation,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    InvocationFailed {
        error: String,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    FileUploaded {
        report_name: String,
        path_in_bucket: String,
        bucket: String,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    FileUploadFailed {
        report_name: String,
        reason: String,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    UploadFailed {
        error: String,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
    DownloadLinkGenerated {
        report_name: String,
        bucket: String,
        path_in_bucket: String,
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
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at
            .expect("entity_first_persisted_at not found")
    }

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

    pub fn last_error(&self) -> Option<String> {
        for e in self.events.iter().rev() {
            if let ReportEvent::CompilationFailed { error, .. } = e {
                return Some(format!("CompilationFailed: {}", error));
            }
            if let ReportEvent::InvocationFailed { error, .. } = e {
                return Some(format!("InvocationFailed: {}", error));
            }
            if let ReportEvent::UploadFailed { error, .. } = e {
                return Some(format!("UploadFailed: {}", error));
            }
            if let ReportEvent::FileUploadFailed { reason, .. } = e {
                return Some(format!("FiledUploadFailed: {}", reason));
            }
        }
        None
    }

    pub fn progress(&self) -> ReportProgress {
        for e in self.events.iter().rev() {
            if let ReportEvent::FileUploaded { .. } = e {
                return ReportProgress::Complete;
            }
        }
        ReportProgress::Running
    }

    pub(super) fn compilation_completed(
        &mut self,
        compilation_result: CompilationResult,
        audit_info: AuditInfo,
    ) {
        self.events.push(ReportEvent::CompilationCompleted {
            result: compilation_result,
            audit_info,
            recorded_at: Utc::now(),
        });
    }

    pub(super) fn compilation_failed(&mut self, error: String, audit_info: AuditInfo) {
        self.events.push(ReportEvent::CompilationFailed {
            error,
            audit_info,
            recorded_at: Utc::now(),
        });
    }

    pub fn compilation_result(&self) -> CompilationResult {
        for e in self.events.iter().rev() {
            if let ReportEvent::CompilationCompleted { result, .. } = e {
                return result.clone();
            }
        }
        unreachable!("Only called after successful compilation");
    }

    pub(super) fn invocation_completed(
        &mut self,
        invocation_result: WorkflowInvocation,
        audit_info: AuditInfo,
    ) {
        self.events.push(ReportEvent::InvocationCompleted {
            result: invocation_result,
            audit_info,
            recorded_at: Utc::now(),
        });
    }

    pub(super) fn invocation_failed(&mut self, error: String, audit_info: AuditInfo) {
        self.events.push(ReportEvent::InvocationFailed {
            error,
            audit_info,
            recorded_at: Utc::now(),
        });
    }

    pub(super) fn files_uploaded(
        &mut self,
        upload_results: Vec<ReportFileUpload>,
        audit_info: AuditInfo,
    ) {
        for res in upload_results {
            match res {
                ReportFileUpload::Success {
                    report_name,
                    path_in_bucket,
                    bucket,
                } => self.events.push(ReportEvent::FileUploaded {
                    report_name,
                    path_in_bucket,
                    bucket,
                    audit_info: audit_info.clone(),
                    recorded_at: Utc::now(),
                }),
                ReportFileUpload::Failure {
                    report_name,
                    reason,
                } => self.events.push(ReportEvent::FileUploadFailed {
                    report_name,
                    reason,
                    audit_info: audit_info.clone(),
                    recorded_at: Utc::now(),
                }),
            }
        }
    }

    pub(super) fn upload_failed(&mut self, error: String, audit_info: AuditInfo) {
        self.events.push(ReportEvent::UploadFailed {
            error,
            audit_info,
            recorded_at: Utc::now(),
        });
    }

    pub(super) fn download_links(&self) -> Vec<ReportLocationInCloud> {
        self.events
            .iter()
            .filter_map(|e| match e {
                ReportEvent::FileUploaded {
                    report_name,
                    bucket,
                    path_in_bucket,
                    ..
                } => Some(ReportLocationInCloud {
                    report_name: report_name.to_string(),
                    bucket: bucket.to_string(),
                    path_in_bucket: path_in_bucket.to_string(),
                }),
                _ => None,
            })
            .collect()
    }

    pub(super) fn download_link_generated(
        &mut self,
        audit_info: AuditInfo,
        location: ReportLocationInCloud,
    ) {
        self.events.push(ReportEvent::DownloadLinkGenerated {
            report_name: location.report_name,
            bucket: location.bucket,
            path_in_bucket: location.path_in_bucket,
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
            if let ReportEvent::Initialized { id, .. } = event {
                builder = builder.id(*id)
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
    use crate::audit::AuditEntryId;

    use super::*;

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
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
            error: "".to_string(),
            audit_info: dummy_audit_info(),
            recorded_at: Utc::now(),
        });
        assert_eq!(
            init_report(events.clone()).next_step(),
            ReportGenerationProcessStep::Compilation
        );

        events.push(ReportEvent::CompilationCompleted {
            result: CompilationResult::default(),
            audit_info: dummy_audit_info(),
            recorded_at: Utc::now(),
        });
        assert_eq!(
            init_report(events.clone()).next_step(),
            ReportGenerationProcessStep::Invocation
        );

        events.push(ReportEvent::InvocationFailed {
            error: "".to_string(),
            audit_info: dummy_audit_info(),
            recorded_at: Utc::now(),
        });
        assert_eq!(
            init_report(events.clone()).next_step(),
            ReportGenerationProcessStep::Invocation
        );

        events.push(ReportEvent::InvocationCompleted {
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
