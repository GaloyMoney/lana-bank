use chrono::{DateTime, Utc};
use derive_builder::Builder;
use es_entity::*;
use serde::{Deserialize, Serialize};

use crate::{audit::AuditInfo, primitives::*, storage::LocationInStorage};

use super::upload::ReportFileUpload;

#[derive(Debug)]
pub struct ReportLocationInCloud {
    pub report_name: String,
    pub bucket: String,
    pub path_in_bucket: String,
}

impl<'a> From<&'a ReportLocationInCloud> for LocationInStorage<'a> {
    fn from(meta: &'a ReportLocationInCloud) -> Self {
        LocationInStorage {
            path: &meta.path_in_bucket,
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

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ReportId")]
pub enum ReportEvent {
    Initialized {
        id: ReportId,
        audit_info: AuditInfo,
    },
    FileUploaded {
        report_name: String,
        path_in_bucket: String,
        bucket: String,
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

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Report {
    pub id: ReportId,
    events: EntityEvents<ReportEvent>,
}

impl Report {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn last_error(&self) -> Option<String> {
        for e in self.events.iter_all().rev() {
            if let ReportEvent::UploadFailed { error, .. } = e {
                return Some(format!("UploadFailed: {}", error));
            }
            if let ReportEvent::UploadFailed { error, .. } = e {
                return Some(format!("FiledUploadFailed: {}", error));
            }
        }
        None
    }

    pub fn progress(&self) -> ReportProgress {
        for e in self.events.iter_all().rev() {
            if let ReportEvent::FileUploaded { .. } = e {
                return ReportProgress::Complete;
            }
        }
        ReportProgress::Running
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
                ReportFileUpload::Failure { reason, .. } => {
                    self.events.push(ReportEvent::UploadFailed {
                        error: reason.to_string(),
                        audit_info: audit_info.clone(),
                        recorded_at: Utc::now(),
                    })
                }
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
            .iter_all()
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

impl TryFromEvents<ReportEvent> for Report {
    fn try_from_events(events: EntityEvents<ReportEvent>) -> Result<Self, EsEntityError> {
        let mut builder = ReportBuilder::default();

        for event in events.iter_all() {
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
}

impl IntoEvents<ReportEvent> for NewReport {
    fn into_events(self) -> EntityEvents<ReportEvent> {
        EntityEvents::init(
            self.id,
            [ReportEvent::Initialized {
                id: self.id,
                audit_info: self.audit_info,
            }],
        )
    }
}
