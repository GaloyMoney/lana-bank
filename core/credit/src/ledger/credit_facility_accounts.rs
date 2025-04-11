use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as CalaAccountId;

use crate::{
    primitives::{LedgerTxId, Satoshis, UsdCents},
    terms::InterestPeriod,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CreditFacilityAccountIds {
    pub facility_account_id: CalaAccountId,
    pub disbursed_receivable_account_id: CalaAccountId,
    pub disbursed_receivable_overdue_account_id: CalaAccountId,
    pub disbursed_defaulted_account_id: CalaAccountId,
    pub collateral_account_id: CalaAccountId,
    pub interest_receivable_account_id: CalaAccountId,
    pub interest_account_id: CalaAccountId,
    pub fee_income_account_id: CalaAccountId,
}

impl CreditFacilityAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            facility_account_id: CalaAccountId::new(),
            disbursed_receivable_account_id: CalaAccountId::new(),
            disbursed_receivable_overdue_account_id: CalaAccountId::new(),
            disbursed_defaulted_account_id: CalaAccountId::new(),
            collateral_account_id: CalaAccountId::new(),
            interest_receivable_account_id: CalaAccountId::new(),
            interest_account_id: CalaAccountId::new(),
            fee_income_account_id: CalaAccountId::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreditFacilityCompletion {
    pub tx_id: LedgerTxId,
    pub collateral: Satoshis,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityActivation {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
    pub debit_account_id: CalaAccountId,
    pub facility_amount: UsdCents,
    pub structuring_fee_amount: UsdCents,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityInterestAccrual {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub interest: UsdCents,
    pub period: InterestPeriod,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
}
