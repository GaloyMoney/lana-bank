#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as CalaAccountId;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct DepositAccountLedgerAccountIds {
    pub deposit_account_id: CalaAccountId,
    pub frozen_deposit_account_id: CalaAccountId,
}

impl DepositAccountLedgerAccountIds {
    pub fn new(account_id: impl Into<CalaAccountId>) -> Self {
        Self {
            deposit_account_id: account_id.into(),
            frozen_deposit_account_id: CalaAccountId::new(),
        }
    }
}
