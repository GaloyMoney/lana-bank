use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use es_entity::*;

use audit::AuditInfo;

use crate::primitives::*;

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "DepositAccountId")]
pub enum DepositAccountEvent {
    Initialized {
        id: DepositAccountId,
        account_holder_id: DepositAccountHolderId,
        ledger_account_id: LedgerAccountId,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct DepositAccount {
    pub id: DepositAccountId,
    pub(super) events: EntityEvents<DepositAccountEvent>,
}

impl TryFromEvents<DepositAccountEvent> for DepositAccount {
    fn try_from_events(events: EntityEvents<DepositAccountEvent>) -> Result<Self, EsEntityError> {
        let mut builder = DepositAccountBuilder::default();
        for event in events.iter_all() {
            match event {
                DepositAccountEvent::Initialized { id, .. } => builder = builder.id(*id),
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewDepositAccount {
    #[builder(setter(into))]
    pub(super) id: DepositAccountId,
    #[builder(setter(into))]
    pub(super) account_holder_id: DepositAccountHolderId,
    #[builder(setter(into))]
    pub audit_info: AuditInfo,
}

impl NewDepositAccount {
    pub fn builder() -> NewDepositAccountBuilder {
        NewDepositAccountBuilder::default()
    }
}

impl IntoEvents<DepositAccountEvent> for NewDepositAccount {
    fn into_events(self) -> EntityEvents<DepositAccountEvent> {
        EntityEvents::init(
            self.id,
            [DepositAccountEvent::Initialized {
                id: self.id,
                account_holder_id: self.account_holder_id,
                ledger_account_id: self.id.into(),
                audit_info: self.audit_info,
            }],
        )
    }
}
