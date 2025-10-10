use derive_builder::Builder;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use es_entity::*;

use crate::primitives::{AnnualClosingTransactionId, CalaTxId};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "AnnualClosingTransactionId")]
pub enum AnnualClosingTransactionEvent {
    Initialized {
        id: AnnualClosingTransactionId,
        ledger_transaction_id: CalaTxId,
        description: String,
        reference: String,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct AnnualClosingTransaction {
    pub id: AnnualClosingTransactionId,
    pub reference: String,
    pub description: String,
    pub ledger_transaction_id: CalaTxId,
    events: EntityEvents<AnnualClosingTransactionEvent>,
}

impl AnnualClosingTransaction {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for annual closing transaction")
    }
}

impl TryFromEvents<AnnualClosingTransactionEvent> for AnnualClosingTransaction {
    fn try_from_events(
        events: EntityEvents<AnnualClosingTransactionEvent>,
    ) -> Result<Self, EsEntityError> {
        let mut builder = AnnualClosingTransactionBuilder::default();
        for event in events.iter_all() {
            match event {
                AnnualClosingTransactionEvent::Initialized {
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
pub struct NewAnnualClosingTransaction {
    #[builder(setter(into))]
    pub(super) id: AnnualClosingTransactionId,
    reference: Option<String>,
    pub(super) ledger_transaction_id: CalaTxId,
    description: String,
}

impl NewAnnualClosingTransaction {
    pub fn builder() -> NewAnnualClosingTransactionBuilder {
        NewAnnualClosingTransactionBuilder::default()
    }

    pub(super) fn reference(&self) -> String {
        match self.reference.as_deref() {
            None => self.id.to_string(),
            Some("") => self.id.to_string(),
            Some(reference) => reference.to_string(),
        }
    }
}

impl IntoEvents<AnnualClosingTransactionEvent> for NewAnnualClosingTransaction {
    fn into_events(self) -> EntityEvents<AnnualClosingTransactionEvent> {
        EntityEvents::init(
            self.id,
            [AnnualClosingTransactionEvent::Initialized {
                reference: self.reference(),
                id: self.id,
                ledger_transaction_id: self.ledger_transaction_id,
                description: self.description,
            }],
        )
    }
}
