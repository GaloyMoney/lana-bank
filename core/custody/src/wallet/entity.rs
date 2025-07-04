use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use crate::primitives::{CustodianId, WalletId};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "WalletId")]
pub enum WalletEvent {
    Initialized {
        id: WalletId,
        custodian_id: CustodianId,
        audit_info: AuditInfo,
    },
    ExternalWalletAttached {
        external_id: String,
        address: String,
        custodian_response: serde_json::Value,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Wallet {
    pub id: WalletId,
    pub custodian_id: CustodianId,

    events: EntityEvents<WalletEvent>,
}

impl Wallet {
    pub fn attach_external_wallet(
        &mut self,
        external_id: String,
        address: String,
        custodian_response: serde_json::Value,
        audit_info: &AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all(),
            WalletEvent::ExternalWalletAttached { external_id: existing, .. } if existing == &external_id
        );

        self.events.push(WalletEvent::ExternalWalletAttached {
            external_id,
            address,
            custodian_response,
            audit_info: audit_info.clone(),
        });

        Idempotent::Executed(())
    }

    pub fn address(&self) -> Option<&str> {
        self.events.iter_all().find_map(|e| match e {
            WalletEvent::ExternalWalletAttached { address, .. } => Some(address.as_str()),
            _ => None,
        })
    }
}

impl TryFromEvents<WalletEvent> for Wallet {
    fn try_from_events(events: EntityEvents<WalletEvent>) -> Result<Self, EsEntityError> {
        let mut builder = WalletBuilder::default();
        for event in events.iter_all() {
            if let WalletEvent::Initialized {
                id, custodian_id, ..
            } = event
            {
                builder = builder.id(*id).custodian_id(*custodian_id);
            }
        }
        builder.events(events).build()
    }
}

#[derive(Debug, Builder)]
pub struct NewWallet {
    #[builder(setter(into))]
    pub(super) id: WalletId,
    #[builder(setter(into))]
    pub(super) custodian_id: CustodianId,
    pub(super) audit_info: AuditInfo,
}

impl NewWallet {
    pub fn builder() -> NewWalletBuilder {
        NewWalletBuilder::default()
    }
}

impl IntoEvents<WalletEvent> for NewWallet {
    fn into_events(self) -> EntityEvents<WalletEvent> {
        EntityEvents::init(
            self.id,
            [WalletEvent::Initialized {
                id: self.id,
                custodian_id: self.custodian_id,
                audit_info: self.audit_info,
            }],
        )
    }
}
