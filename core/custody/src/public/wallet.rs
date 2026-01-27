use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use core_money::Satoshis;

use crate::{
    primitives::{WalletId, WalletNetwork},
    wallet::Wallet,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicWallet {
    pub id: WalletId,
    pub address: String,
    pub network: WalletNetwork,
    pub balance: Option<Satoshis>,
    pub balance_updated_at: Option<DateTime<Utc>>,
}

impl From<&Wallet> for PublicWallet {
    fn from(entity: &Wallet) -> Self {
        PublicWallet {
            id: entity.id,
            address: entity.address.clone(),
            network: entity.network,
            balance: entity.current_balance(),
            balance_updated_at: entity.last_balance_update(),
        }
    }
}
