use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    primitives::{WalletId, WalletNetwork},
    wallet::{Wallet, WalletBalance},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicWallet {
    pub id: WalletId,
    pub address: String,
    pub network: WalletNetwork,
    pub balance: Option<WalletBalance>,
}

impl From<&Wallet> for PublicWallet {
    fn from(entity: &Wallet) -> Self {
        PublicWallet {
            id: entity.id,
            address: entity.address.clone(),
            network: entity.network,
            balance: entity.balance(),
        }
    }
}
