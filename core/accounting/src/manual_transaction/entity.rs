use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::{CalaTxId, ManualTransactionId};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "ManualTransactionId")]
pub enum ManualTransactionEvent {
    Initialized {
        id: ManualTransactionId,
        ledger_transaction_id: CalaTxId,
        description: String,
        reference: String,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct ManualTransaction {
    pub id: ManualTransactionId,
    pub reference: String,
    pub description: String,
    pub ledger_transaction_id: CalaTxId,
    events: EntityEvents<ManualTransactionEvent>,
}

impl ManualTransaction {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for deposit")
    }
}

impl TryFromEvents<ManualTransactionEvent> for ManualTransaction {
    fn try_from_events(
        events: EntityEvents<ManualTransactionEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = ManualTransactionBuilder::default();
        for event in events.iter_all() {
            match event {
                ManualTransactionEvent::Initialized {
                    id,
                    reference,
                    description,
                    ledger_transaction_id,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .reference(reference.clone())
                        .description(description.clone())
                        .ledger_transaction_id(*ledger_transaction_id)
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewManualTransaction {
    #[builder(setter(into))]
    pub(super) id: ManualTransactionId,
    reference: Option<String>,
    pub(super) ledger_transaction_id: CalaTxId,
    description: String,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewManualTransaction {
    pub fn builder() -> NewManualTransactionBuilder {
        NewManualTransactionBuilder::default()
    }

    pub(super) fn reference(&self) -> String {
        match self.reference.as_deref() {
            None => self.id.to_string(),
            Some("") => self.id.to_string(),
            Some(reference) => reference.to_string(),
        }
    }
}

impl IntoEvents<ManualTransactionEvent> for NewManualTransaction {
    fn into_events(self) -> EntityEvents<ManualTransactionEvent> {
        EntityEvents::init(
            self.id,
            [ManualTransactionEvent::Initialized {
                reference: self.reference(),
                id: self.id,
                ledger_transaction_id: self.ledger_transaction_id,
                description: self.description,
                audit_info: self.audit_info,
            }],
        )
    }
}
