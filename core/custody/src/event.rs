use serde::{Deserialize, Serialize};

use crate::primitives::WalletId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreCustodyEvent {
    WalletAddressAllocated {
        id: WalletId,
        label: String,
        address: String,
    },
}
