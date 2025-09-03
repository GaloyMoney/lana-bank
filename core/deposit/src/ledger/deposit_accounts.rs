#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as CalaAccountId;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct DepositAccountLedgerAccountIds {
    // Ledger account ID holding the deposit for this account.
    pub ledger_account_id: CalaAccountId,
    // Ledger account ID potentially holding frozen deposit from this account.
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
    pub fn with_ledger_account(ledger_account_id: impl Into<CalaAccountId>) -> Self {
        Self {
            ledger_account_id: ledger_account_id.into(),
            frozen_deposit_account_id: CalaAccountId::new(),
        }
    }
}
