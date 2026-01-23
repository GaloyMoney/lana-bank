use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use super::primitives::{DepositAccountHolderId, DepositAccountId, DepositId, WithdrawalId};
use core_money::UsdCents;

#[derive(Debug, Serialize, Deserialize, strum::AsRefStr)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum CoreDepositEvent {
    DepositAccountCreated {
        id: DepositAccountId,
        account_holder_id: DepositAccountHolderId,
    },
    DepositInitialized {
        id: DepositId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
    },
    WithdrawalConfirmed {
        id: WithdrawalId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
    },
    DepositReverted {
        id: DepositId,
        deposit_account_id: DepositAccountId,
        amount: UsdCents,
    },
    DepositAccountFrozen {
        id: DepositAccountId,
        account_holder_id: DepositAccountHolderId,
    },
}
