use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    deposit::Deposit,
    primitives::{DepositAccountId, DepositId, UsdCents},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicDeposit {
    pub id: DepositId,
    pub deposit_account_id: DepositAccountId,
    pub amount: UsdCents,
}

impl From<&Deposit> for PublicDeposit {
    fn from(entity: &Deposit) -> Self {
        PublicDeposit {
            id: entity.id,
            deposit_account_id: entity.deposit_account_id,
            amount: entity.amount,
        }
    }
}
