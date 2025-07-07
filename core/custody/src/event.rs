use serde::{Deserialize, Serialize};

use crate::primitives::WalletId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreCustodyEvent {
    WalletAttached { id: WalletId, address: String },
}
