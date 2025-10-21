#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as CalaAccountId;

use crate::{
    primitives::{CreditFacilityId, CustomerType, LedgerTxId, Satoshis, UsdCents},
    terms::{FacilityDurationType, InterestPeriod},
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CreditFacilityLedgerAccountIds {
    pub facility_account_id: CalaAccountId,
    pub in_liquidation_account_id: CalaAccountId,
    pub disbursed_receivable_not_yet_due_account_id: CalaAccountId,
    pub disbursed_receivable_due_account_id: CalaAccountId,
    pub disbursed_receivable_overdue_account_id: CalaAccountId,
    pub disbursed_defaulted_account_id: CalaAccountId,
    pub collateral_account_id: CalaAccountId,
    pub interest_receivable_not_yet_due_account_id: CalaAccountId,
    pub interest_receivable_due_account_id: CalaAccountId,
    pub interest_receivable_overdue_account_id: CalaAccountId,
    pub interest_defaulted_account_id: CalaAccountId,
    pub interest_income_account_id: CalaAccountId,
    pub fee_income_account_id: CalaAccountId,
}

impl CreditFacilityLedgerAccountIds {
    #[allow(clippy::new_without_default)]
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            facility_account_id: CalaAccountId::new(),
            collateral_account_id: CalaAccountId::new(),
            in_liquidation_account_id: CalaAccountId::new(),
            disbursed_receivable_not_yet_due_account_id: CalaAccountId::new(),
            disbursed_receivable_due_account_id: CalaAccountId::new(),
            disbursed_receivable_overdue_account_id: CalaAccountId::new(),
            disbursed_defaulted_account_id: CalaAccountId::new(),
            interest_receivable_not_yet_due_account_id: CalaAccountId::new(),
            interest_receivable_due_account_id: CalaAccountId::new(),
            interest_receivable_overdue_account_id: CalaAccountId::new(),
            interest_defaulted_account_id: CalaAccountId::new(),
            interest_income_account_id: CalaAccountId::new(),
            fee_income_account_id: CalaAccountId::new(),
        }
    }
}

impl From<PendingCreditFacilityAccountIds> for CreditFacilityLedgerAccountIds {
    fn from(proposal_ids: PendingCreditFacilityAccountIds) -> Self {
        Self {
            facility_account_id: proposal_ids.facility_account_id,
            collateral_account_id: proposal_ids.collateral_account_id,
            in_liquidation_account_id: CalaAccountId::new(),
            disbursed_receivable_not_yet_due_account_id: CalaAccountId::new(),
            disbursed_receivable_due_account_id: CalaAccountId::new(),
            disbursed_receivable_overdue_account_id: CalaAccountId::new(),
            disbursed_defaulted_account_id: CalaAccountId::new(),
            interest_receivable_not_yet_due_account_id: CalaAccountId::new(),
            interest_receivable_due_account_id: CalaAccountId::new(),
            interest_receivable_overdue_account_id: CalaAccountId::new(),
            interest_defaulted_account_id: CalaAccountId::new(),
            interest_income_account_id: CalaAccountId::new(),
            fee_income_account_id: CalaAccountId::new(),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PendingCreditFacilityAccountIds {
    pub facility_account_id: CalaAccountId,
    pub collateral_account_id: CalaAccountId,
}

impl PendingCreditFacilityAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            collateral_account_id: CalaAccountId::new(),
            facility_account_id: CalaAccountId::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreditFacilityCompletion {
    pub tx_id: LedgerTxId,
    pub collateral: Satoshis,
    pub credit_facility_account_ids: CreditFacilityLedgerAccountIds,
}

#[derive(Debug, Clone)]
pub struct PendingCreditFacilityCreation {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub pending_credit_facility_account_ids: PendingCreditFacilityAccountIds,
    pub facility_amount: UsdCents,
}

pub struct CreditFacilityActivation {
    pub credit_facility_id: CreditFacilityId,
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub account_ids: CreditFacilityLedgerAccountIds,
    pub customer_type: CustomerType,
    pub duration_type: FacilityDurationType,
    pub facility_amount: UsdCents,
    pub debit_account_id: CalaAccountId,
    pub structuring_fee_amount: UsdCents,
    pub principal_amount: UsdCents,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityInterestAccrual {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub interest: UsdCents,
    pub period: InterestPeriod,
    pub credit_facility_account_ids: CreditFacilityLedgerAccountIds,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityInterestAccrualCycle {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub interest: UsdCents,
    pub effective: chrono::NaiveDate,
    pub credit_facility_account_ids: CreditFacilityLedgerAccountIds,
}
