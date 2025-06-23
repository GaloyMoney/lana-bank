use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use cloud_storage::LocationInStorage;
use es_entity::*;

use crate::csv::primitives::{AccountingCsvStatus, AccountingCsvType};
use crate::primitives::{AccountingCsvId, LedgerAccountId};

use super::error::AccountingCsvError;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "AccountingCsvId")]
pub enum AccountingCsvEvent {
    Initialized {
        id: AccountingCsvId,
        csv_type: AccountingCsvType,
        ledger_account_id: Option<LedgerAccountId>,
        path_in_bucket: String,
        audit_info: AuditInfo,
    },
    FileUploaded {
        bucket: String,
        recorded_at: DateTime<Utc>,
    },
    UploadFailed {
        error: String,
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
    pub(super) path_in_storage: String,
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

    pub fn file_uploaded(&mut self, bucket: String) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            AccountingCsvEvent::FileUploaded { .. }
        );

        self.events.push(AccountingCsvEvent::FileUploaded {
            bucket,
            recorded_at: Utc::now(),
        });
        Idempotent::Executed(())
    }

    pub fn upload_failed(&mut self, error: String) {
        self.events.push(AccountingCsvEvent::UploadFailed {
            error,
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

    pub fn path_in_bucket(&self) -> &str {
        &self.path_in_storage
    }

    pub fn download_link_generated(
        &mut self,
        audit_info: AuditInfo,
    ) -> Result<LocationInStorage, AccountingCsvError> {
        if self.status() != AccountingCsvStatus::Completed {
            return Err(AccountingCsvError::CsvNotReady);
        }
        let paths = self
            .events
            .iter_all()
            .rev()
            .find_map(|e| {
                if let AccountingCsvEvent::FileUploaded { bucket, .. } = e {
                    Some(bucket.to_string())
                } else {
                    None
                }
            })
            .expect("paths not found");
        self.events.push(AccountingCsvEvent::DownloadLinkGenerated {
            bucket: paths,
            path_in_bucket: self.path_in_storage.clone(),
            audit_info,
            recorded_at: Utc::now(),
        });

        Ok(LocationInStorage {
            path: &self.path_in_storage,
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
                path_in_bucket,
                ..
            } = event
            {
                builder = builder.id(*id).csv_type(*csv_type);
                builder = builder.path_in_storage(path_in_bucket.clone());
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
                path_in_bucket: format!("accounting_csvs/{}.csv", self.id),
                audit_info: self.audit_info,
            }],
        )
    }
}
