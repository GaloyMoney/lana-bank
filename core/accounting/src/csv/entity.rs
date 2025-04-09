use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::{AccountingCsvId, LedgerAccountId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AccountingCsvType {
    LedgerAccount { ledger_account_id: LedgerAccountId },
}

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "AccountingCsvId")]
pub enum AccountingCsvEvent {
    Initialized {
        id: AccountingCsvId,
        csv_type: AccountingCsvType,
        audit_info: AuditInfo,
    },
    FileUploaded {
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
        bucket: String,
        path_in_bucket: String,
        audit_info: AuditInfo,
        recorded_at: DateTime<Utc>,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct AccountingCsv {
    pub id: AccountingCsvId,
    pub(super) events: EntityEvents<AccountingCsvEvent>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountingCsvStatus {
    Pending,
    Completed,
    Failed,
}

impl AccountingCsv {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn csv_type(&self) -> AccountingCsvType {
        for e in self.events.iter_all() {
            if let AccountingCsvEvent::Initialized { csv_type, .. } = e {
                return csv_type.clone();
            }
        }
    }

    pub fn status(&self) -> AccountingCsvStatus {
        for e in self.events.iter_all().rev() {
            match e {
                AccountingCsvEvent::FileUploaded { .. } => return AccountingCsvStatus::Completed,
                AccountingCsvEvent::UploadFailed { .. } => return AccountingCsvStatus::Failed,
                _ => {}
            }
        }
        AccountingCsvStatus::Pending
    }

    pub fn last_error(&self) -> Option<String> {
        for e in self.events.iter_all().rev() {
            if let AccountingCsvEvent::UploadFailed { error, .. } = e {
                return Some(error.clone());
            }
        }
        None
    }

    pub fn file_uploaded(&mut self, path_in_bucket: String, bucket: String, audit_info: AuditInfo) {
        self.events.push(AccountingCsvEvent::FileUploaded {
            path_in_bucket,
            bucket,
            audit_info,
            recorded_at: Utc::now(),
        });
    }

    pub fn upload_failed(&mut self, error: String, audit_info: AuditInfo) {
        self.events.push(AccountingCsvEvent::UploadFailed {
            error,
            audit_info,
            recorded_at: Utc::now(),
        });
    }

    pub fn download_link_generated(
        &mut self,
        bucket: String,
        path_in_bucket: String,
        audit_info: AuditInfo,
    ) {
        self.events.push(AccountingCsvEvent::DownloadLinkGenerated {
            bucket,
            path_in_bucket,
            audit_info,
            recorded_at: Utc::now(),
        });
    }
}

impl TryFromEvents<AccountingCsvEvent> for AccountingCsv {
    fn try_from_events(events: EntityEvents<AccountingCsvEvent>) -> Result<Self, EsEntityError> {
        let mut builder = AccountingCsvBuilder::default();

        for event in events.iter_all() {
            match event {
                AccountingCsvEvent::Initialized { id, .. } => {
                    builder = builder.id(*id);
                }
                _ => {}
            }
        }

        builder.events(events).build()
    }
}

#[derive(Builder)]
pub struct NewAccountingCsv {
    #[builder(setter(into))]
    pub(super) id: AccountingCsvId,
    #[builder(setter(into))]
    pub(super) csv_type: AccountingCsvType,
    #[builder(setter(into))]
    pub(super) audit_info: AuditInfo,
}

impl NewAccountingCsv {
    pub fn builder() -> NewAccountingCsvBuilder {
        NewAccountingCsvBuilder::default()
    }
}

impl IntoEvents<AccountingCsvEvent> for NewAccountingCsv {
    fn into_events(self) -> EntityEvents<AccountingCsvEvent> {
        EntityEvents::init(
            self.id,
            [AccountingCsvEvent::Initialized {
                id: self.id,
                csv_type: self.csv_type,
                audit_info: self.audit_info,
            }],
        )
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedAccountingCsvDownloadLink {
    pub accounting_csv_id: AccountingCsvId,
    pub link: String,
}
