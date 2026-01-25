use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

use crate::{
    account::DepositAccount,
    primitives::{DepositAccountHolderId, DepositAccountId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PublicDepositAccount {
    pub id: DepositAccountId,
    pub account_holder_id: DepositAccountHolderId,
}

impl From<&DepositAccount> for PublicDepositAccount {
    fn from(entity: &DepositAccount) -> Self {
        PublicDepositAccount {
            id: entity.id,
            account_holder_id: entity.account_holder_id,
        }
    }
}
