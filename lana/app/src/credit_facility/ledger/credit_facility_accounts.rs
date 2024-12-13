use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as LedgerAccountId;

use crate::primitives::{Satoshis, UsdCents};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CreditFacilityAccountIds {
    pub facility_account_id: LedgerAccountId,
    pub disbursed_receivable_account_id: LedgerAccountId,
    pub collateral_account_id: LedgerAccountId,
    pub interest_receivable_account_id: LedgerAccountId,
    pub interest_account_id: LedgerAccountId,
    pub fee_income_account_id: LedgerAccountId,
}

impl CreditFacilityAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            facility_account_id: LedgerAccountId::new(),
            disbursed_receivable_account_id: LedgerAccountId::new(),
            collateral_account_id: LedgerAccountId::new(),
            interest_receivable_account_id: LedgerAccountId::new(),
            interest_account_id: LedgerAccountId::new(),
            fee_income_account_id: LedgerAccountId::new(),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CreditFacilityLedgerBalance {
    pub facility: UsdCents,
    pub collateral: Satoshis,
    pub disbursed: UsdCents,
    pub disbursed_receivable: UsdCents,
    pub interest: UsdCents,
    pub interest_receivable: UsdCents,
}

impl CreditFacilityLedgerBalance {
    pub fn check_disbursal_amount(&self, amount: UsdCents) -> bool {
        amount < self.facility
    }
}
