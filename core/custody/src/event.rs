use serde::{Deserialize, Serialize};

use core_money::Satoshis;

use crate::primitives::WalletId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreCustodyEvent {
    WalletAttached {
        id: WalletId,
        address: String,
    },
    WalletBalanceChanged {
        external_wallet_id: String,
        amount: Satoshis,
    },
}
