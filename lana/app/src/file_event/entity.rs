use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::accounting::csv::AccountingCsvType;
use crate::accounting::LedgerAccountId;
use crate::audit::AuditInfo;
use crate::primitives::{CustomerId, ReportId};

use es_entity::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    Document,
    Report,
    AccountingCsv,
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "Uuid")]
pub enum FileEvent {
    Initialized {
        /// Identifier for the file entity
        id: Uuid,
        /// Which file category this event relates to
        file_kind: FileKind,
        audit_info: AuditInfo,
        // Document specific metadata
        #[serde(skip_serializing_if = "Option::is_none")]
        customer_id: Option<CustomerId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sanitized_filename: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        original_filename: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        path_in_bucket: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        bucket: Option<String>,
        // AccountingCsv specifics
        #[serde(skip_serializing_if = "Option::is_none")]
        csv_type: Option<AccountingCsvType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ledger_account_id: Option<LedgerAccountId>,
        // Report specifics
        #[serde(skip_serializing_if = "Option::is_none")]
        report_id: Option<ReportId>,
    },
    Uploaded {
        file_kind: FileKind,
        audit_info: AuditInfo,
        path_in_bucket: String,
        bucket: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        report_name: Option<String>,
        recorded_at: DateTime<Utc>,
    },
    UploadFailed {
        file_kind: FileKind,
        audit_info: AuditInfo,
        error: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        report_name: Option<String>,
        recorded_at: DateTime<Utc>,
    },
    Deleted {
        file_kind: FileKind,
        audit_info: AuditInfo,
    },
    Archived {
        file_kind: FileKind,
        audit_info: AuditInfo,
    },
    DownloadLinkCreated {
        file_kind: FileKind,
        audit_info: AuditInfo,
        bucket: String,
        path_in_bucket: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        report_name: Option<String>,
        recorded_at: DateTime<Utc>,
    },
}
