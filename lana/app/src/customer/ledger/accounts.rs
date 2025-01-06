use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as LedgerAccountId;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CustomerAccountIds {
    pub deposit_account_id: LedgerAccountId,
}

impl CustomerAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            deposit_account_id: LedgerAccountId::new(),
        }
    }
}
