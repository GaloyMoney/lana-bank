use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use core_money::Satoshis;

use crate::primitives::WalletId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreCustodyEvent {
    WalletBalanceChanged {
        id: WalletId,
        new_balance: Satoshis,
        changed_at: DateTime<Utc>,
    },
}
