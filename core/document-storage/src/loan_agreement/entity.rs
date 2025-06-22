use chrono::{DateTime, Utc};
use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::{CustomerId};
use super::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "LoanAgreementId")]
pub enum LoanAgreementEvent {
    Initialized {
        id: LoanAgreementId,
        customer_id: CustomerId,
        audit_info: AuditInfo,
    },
    FileGenerated {
        storage_path: String,
        filename: String,
        audit_info: AuditInfo,
    },
    GenerationFailed {
        error: String,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct LoanAgreement {
    pub id: LoanAgreementId,
    pub customer_id: CustomerId,
    pub status: LoanAgreementStatus,
    pub storage_path: Option<String>,
    pub filename: Option<String>,
    pub error_message: Option<String>,
    events: EntityEvents<LoanAgreementEvent>,
}

impl core::fmt::Display for LoanAgreement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LoanAgreement: {}", self.id)
    }
}

impl LoanAgreement {
    pub fn created_at(&self) -> DateTime<Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("entity_first_persisted_at not found")
    }

    pub fn file_generated(
        &mut self,
        storage_path: String,
        filename: String,
        audit_info: AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(self.events.iter_all(), LoanAgreementEvent::FileGenerated { .. });

        self.events.push(LoanAgreementEvent::FileGenerated {
            storage_path: storage_path.clone(),
            filename: filename.clone(),
            audit_info,
        });
        
        self.status = LoanAgreementStatus::Completed;
        self.storage_path = Some(storage_path);
        self.filename = Some(filename);
        
        Idempotent::Executed(())
    }

    pub fn generation_failed(&mut self, error: String, audit_info: AuditInfo) -> Idempotent<()> {
        idempotency_guard!(self.events.iter_all(), LoanAgreementEvent::GenerationFailed { .. });

        self.events.push(LoanAgreementEvent::GenerationFailed {
            error: error.clone(),
            audit_info,
        });
        
        self.status = LoanAgreementStatus::Failed;
        self.error_message = Some(error);
        
        Idempotent::Executed(())
    }
}

impl TryFromEvents<LoanAgreementEvent> for LoanAgreement {
    fn try_from_events(events: EntityEvents<LoanAgreementEvent>) -> Result<Self, EsEntityError> {
        let mut builder = LoanAgreementBuilder::default();
        
        let mut status = LoanAgreementStatus::Pending;
        let mut storage_path = None;
        let mut filename = None;
        let mut error_message = None;

        for event in events.iter_all() {
            match event {
                LoanAgreementEvent::Initialized { id, customer_id, .. } => {
                    builder = builder.id(*id).customer_id(*customer_id);
                }
                LoanAgreementEvent::FileGenerated { storage_path: path, filename: fname, .. } => {
                    status = LoanAgreementStatus::Completed;
                    storage_path = Some(path.clone());
                    filename = Some(fname.clone());
                }
                LoanAgreementEvent::GenerationFailed { error, .. } => {
                    status = LoanAgreementStatus::Failed;
                    error_message = Some(error.clone());
                }
            }
        }

        builder
            .status(status)
            .storage_path(storage_path)
            .filename(filename)
            .error_message(error_message)
            .events(events)
            .build()
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct NewLoanAgreement {
    #[builder(setter(into))]
    pub(super) id: LoanAgreementId,
    #[builder(setter(into))]
    pub(super) customer_id: CustomerId,
    pub(super) audit_info: AuditInfo,
}

impl NewLoanAgreement {
    pub fn builder() -> NewLoanAgreementBuilder {
        NewLoanAgreementBuilder::default()
    }
}

impl IntoEvents<LoanAgreementEvent> for NewLoanAgreement {
    fn into_events(self) -> EntityEvents<LoanAgreementEvent> {
        EntityEvents::init(
            self.id,
            [LoanAgreementEvent::Initialized {
                id: self.id,
                customer_id: self.customer_id,
                audit_info: self.audit_info,
            }],
        )
    }
}