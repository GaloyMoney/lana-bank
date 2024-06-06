use crate::primitives::LedgerAccountId;
use serde::{Deserialize, Serialize};

use super::constants;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BankLedgerAccountIds {
    pub shareholder_equity_id: LedgerAccountId,
}

impl Default for BankLedgerAccountIds {
    fn default() -> Self {
        Self {
            shareholder_equity_id: LedgerAccountId::from(constants::BANK_SHAREHOLDER_EQUITY_ID),
        }
    }
}
