use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as LedgerAccountId;

use crate::{
    primitives::{LedgerTxId, Satoshis, UsdCents},
    terms::InterestPeriod,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct CreditFacilityPaymentAmounts {
    pub interest: UsdCents,
    pub disbursal: UsdCents,
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
    pub debit_account_id: LedgerAccountId,
    pub facility_amount: UsdCents,
    pub structuring_fee_amount: UsdCents,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityInterestIncurrence {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub interest: UsdCents,
    pub period: InterestPeriod,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityInterestAccrual {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub interest: UsdCents,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
    pub accrued_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DisbursalData {
    pub amount: UsdCents,
    pub tx_ref: String,
    pub tx_id: LedgerTxId,
    pub cancelled: bool,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
    pub debit_account_id: LedgerAccountId,
}
