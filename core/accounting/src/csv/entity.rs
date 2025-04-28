use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::csv::primitives::{
    AccountingCsvLocationInCloud, AccountingCsvStatus, AccountingCsvType,
};
use crate::primitives::{AccountingCsvId, LedgerAccountId};

use super::error::AccountingCsvError;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "AccountingCsvId")]
pub enum AccountingCsvEvent {
    Initialized {
        id: AccountingCsvId,
        csv_type: AccountingCsvType,
        ledger_account_id: Option<LedgerAccountId>,
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
    pub csv_type: AccountingCsvType,
    #[builder(setter(strip_option), default)]
    pub ledger_account_id: Option<LedgerAccountId>,
    events: EntityEvents<AccountingCsvEvent>,
}

impl AccountingCsv {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
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

    pub fn last_error(&self) -> Option<&str> {
        for e in self.events.iter_all().rev() {
            if let AccountingCsvEvent::UploadFailed { error, .. } = e {
                return Some(error);
            }
        }
        None
    }

    pub fn file_uploaded(
        &mut self,
        path_in_bucket: String,
        bucket: String,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            AccountingCsvEvent::FileUploaded { .. }
        );

        self.events.push(AccountingCsvEvent::FileUploaded {
            path_in_bucket,
            bucket,
            audit_info,
            recorded_at: Utc::now(),
        });
        Idempotent::Executed(())
    }

    pub fn upload_failed(&mut self, error: String, audit_info: AuditInfo) {
        self.events.push(AccountingCsvEvent::UploadFailed {
            error,
            audit_info,
            recorded_at: Utc::now(),
        });
    }

    pub fn bucket(&self) -> Option<&str> {
        for e in self.events.iter_all().rev() {
            if let AccountingCsvEvent::FileUploaded { bucket, .. } = e {
                return Some(bucket);
            }
        }
        None
    }

    pub fn path_in_bucket(&self) -> Option<&str> {
        for e in self.events.iter_all().rev() {
            if let AccountingCsvEvent::FileUploaded { path_in_bucket, .. } = e {
                return Some(path_in_bucket);
            }
        }
        None
    }

    pub fn download_link_generated(
        &mut self,
        audit_info: AuditInfo,
    ) -> Result<AccountingCsvLocationInCloud, AccountingCsvError> {
        if self.status() != AccountingCsvStatus::Completed {
            return Err(AccountingCsvError::CsvNotReady);
        }
        let paths = self.events.iter_all().rev().find_map(|e| {
            if let AccountingCsvEvent::FileUploaded {
                bucket,
                path_in_bucket,
                ..
            } = e
            {
                Some((bucket.to_string(), path_in_bucket.to_string()))
            } else {
                None
            }
        });
        let (bucket, path_in_bucket) = paths.ok_or(AccountingCsvError::CsvFileNotFound)?;
        self.events.push(AccountingCsvEvent::DownloadLinkGenerated {
            bucket,
            path_in_bucket,
            audit_info,
            recorded_at: Utc::now(),
        });
        let paths = self
            .events
            .iter_all()
            .rev()
            .find_map(|e| {
                if let AccountingCsvEvent::FileUploaded {
                    bucket,
                    path_in_bucket,
                    ..
                } = e
                {
                    Some((bucket, path_in_bucket))
                } else {
                    None
                }
            })
            .expect("path exists");

        Ok(AccountingCsvLocationInCloud {
            csv_type: self.csv_type,
            bucket: paths.0,
            path_in_bucket: paths.1,
        })
    }
}

impl TryFromEvents<AccountingCsvEvent> for AccountingCsv {
    fn try_from_events(events: EntityEvents<AccountingCsvEvent>) -> Result<Self, EsEntityError> {
        let mut builder = AccountingCsvBuilder::default();

        for event in events.iter_all() {
            if let AccountingCsvEvent::Initialized {
                id,
                csv_type,
                ledger_account_id,
                ..
            } = event
            {
                builder = builder.id(*id).csv_type(*csv_type);
                if let Some(account_id) = ledger_account_id {
                    builder = builder.ledger_account_id(*account_id);
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Builder, Debug)]
pub struct NewAccountingCsv {
    #[builder(setter(into))]
    pub(super) id: AccountingCsvId,
    #[builder(setter(into))]
    pub(super) csv_type: AccountingCsvType,
    #[builder(setter(strip_option), default)]
    pub(super) ledger_account_id: Option<LedgerAccountId>,
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
                ledger_account_id: self.ledger_account_id,
                audit_info: self.audit_info,
            }],
        )
    }
}
