use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use audit::AuditInfo;

use crate::primitives::{DepositAccountId, DepositId};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DepositId")]
pub enum DepositEvent {
    Initialized {
        id: DepositId,
        deposit_account_id: DepositAccountId,
        reference: String,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Deposit {
    pub id: DepositId,
    pub deposit_account_id: DepositAccountId,
    pub reference: String,
    pub(super) events: EntityEvents<DepositEvent>,
}

impl Deposit {
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.events
            .entity_first_persisted_at()
            .expect("No events for deposit")
    }
}

impl TryFromEvents<DepositEvent> for Deposit {
    fn try_from_events(events: EntityEvents<DepositEvent>) -> Result<Self, EsEntityError> {
        let mut builder = DepositBuilder::default();
        for event in events.iter_all() {
            match event {
                DepositEvent::Initialized {
                    id,
                    reference,
                    deposit_account_id,
                    ..
                } => {
                    builder = builder
                        .id(*id)
                        .deposit_account_id(*deposit_account_id)
                        .reference(reference.clone());
                }
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewDeposit {
    #[builder(setter(into))]
    pub(super) id: DepositId,
    #[builder(setter(into))]
    pub(super) deposit_account_id: DepositAccountId,
    reference: Option<String>,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewDeposit {
    pub fn builder() -> NewDepositBuilder {
        NewDepositBuilder::default()
    }

    pub(super) fn reference(&self) -> String {
        match self.reference.as_deref() {
            None => self.id.to_string(),
            Some("") => self.id.to_string(),
            Some(reference) => reference.to_string(),
        }
    }
}

impl IntoEvents<DepositEvent> for NewDeposit {
    fn into_events(self) -> EntityEvents<DepositEvent> {
        EntityEvents::init(
            self.id,
            [DepositEvent::Initialized {
                reference: self.reference(),
                id: self.id,
                deposit_account_id: self.deposit_account_id,
                audit_info: self.audit_info,
            }],
        )
    }
}
