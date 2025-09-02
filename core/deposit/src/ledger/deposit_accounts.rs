#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as CalaAccountId;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct DepositAccountLedgerAccountIds {
    pub ledger_account_id: CalaAccountId,
    pub frozen_deposit_account_id: CalaAccountId,
}

impl DepositAccountLedgerAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            ledger_account_id: CalaAccountId::new(),
            frozen_deposit_account_id: CalaAccountId::new(),
        }
    }
}
