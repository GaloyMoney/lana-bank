use serde::{Deserialize, Serialize};

use super::primitives::{
    DepositAccountHolderId, DepositAccountId, DepositId, DepositStatus, WithdrawalId,
};
use core_money::UsdCents;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CoreDepositEvent {
    DepositAccountCreated {
        id: DepositAccountId,
        account_holder_id: DepositAccountHolderId,
    },
    DepositStatusUpdated {
        id: DepositId,
        status: DepositStatus,
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
}
