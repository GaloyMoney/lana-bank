use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    primitives::{DepositAccountId, WithdrawalId},
    withdrawal::{Withdrawal, WithdrawalStatus},
};

use money::UntypedAmount;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicWithdrawal {
    pub id: WithdrawalId,
    pub deposit_account_id: DepositAccountId,
    pub amount: UntypedAmount,
    pub status: WithdrawalStatus,
}

impl From<&Withdrawal> for PublicWithdrawal {
    fn from(entity: &Withdrawal) -> Self {
        PublicWithdrawal {
            id: entity.id,
            deposit_account_id: entity.deposit_account_id,
            amount: entity.amount,
            status: entity.status(),
        }
    }
}
