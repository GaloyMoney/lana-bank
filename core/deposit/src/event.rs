use serde::{Deserialize, Serialize};

use super::primitives::{DepositAccountHolderId, DepositAccountId, DepositId, WithdrawalId};
use core_money::UsdCents;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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
}
