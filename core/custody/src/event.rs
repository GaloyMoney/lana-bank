use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use core_money::Satoshis;

use crate::primitives::WalletId;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreCustodyEvent {
    WalletBalanceChanged {
        id: WalletId,
        new_balance: Satoshis,
        changed_at: DateTime<Utc>,
    },
}
