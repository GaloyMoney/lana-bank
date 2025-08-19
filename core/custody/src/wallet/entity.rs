use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use es_entity::*;

use core_money::Satoshis;

use crate::primitives::{CustodianId, WalletId};

#[derive(EsEvent, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[es_event(id = "WalletId")]
pub enum WalletEvent {
    Initialized {
        id: WalletId,
        custodian_id: CustodianId,
        external_wallet_id: String,
        address: String,
        custodian_response: serde_json::Value,
        audit_info: AuditInfo,
    },
    BalanceChanged {
        new_balance: Satoshis,
        changed_at: DateTime<Utc>,
        audit_info: AuditInfo,
    },
}

#[derive(EsEntity, Builder)]
#[builder(pattern = "owned", build_fn(error = "EsEntityError"))]
pub struct Wallet {
    pub id: WalletId,
    pub custodian_id: CustodianId,
    pub address: String,
    pub external_wallet_id: String,

    events: EntityEvents<WalletEvent>,
}

impl Wallet {
    pub fn update_balance(
        &mut self,
        new_balance: Satoshis,
        update_time: DateTime<Utc>,
        audit_info: &AuditInfo,
    ) -> Idempotent<()> {
        idempotency_guard!(
            self.events.iter_all().rev(),
            WalletEvent::BalanceChanged { new_balance: balance, .. } if *balance == new_balance ,
            => WalletEvent::BalanceChanged { .. }
        );

        self.events.push(WalletEvent::BalanceChanged {
            new_balance,
            changed_at: update_time,
            audit_info: audit_info.clone(),
        });

        Idempotent::Executed(())
    }
}

impl TryFromEvents<WalletEvent> for Wallet {
    fn try_from_events(events: EntityEvents<WalletEvent>) -> Result<Self, EsEntityError> {
        let mut builder = WalletBuilder::default();
        for event in events.iter_all() {
            if let WalletEvent::Initialized {
                id,
                custodian_id,
                address,
                external_wallet_id,
                ..
            } = event
            {
                builder = builder
                    .id(*id)
                    .custodian_id(*custodian_id)
                    .address(address.to_owned())
                    .external_wallet_id(external_wallet_id.to_owned());
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
    pub(super) custodian_response: serde_json::Value,
    pub(super) address: String,
    pub(super) external_wallet_id: String,
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
                external_wallet_id: self.external_wallet_id,
                address: self.address,
                custodian_response: self.custodian_response,
            }],
        )
    }
}
