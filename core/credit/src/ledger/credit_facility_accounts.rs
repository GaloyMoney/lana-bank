#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as CalaAccountId;

use crate::{
    primitives::{CreditFacilityId, CreditFacilityProposalId, LedgerTxId, Satoshis, UsdCents},
    terms::InterestPeriod,
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
    pub fn new() -> Self {
        Self {
            facility_account_id: CalaAccountId::new(),
            in_liquidation_account_id: CalaAccountId::new(),
            disbursed_receivable_not_yet_due_account_id: CalaAccountId::new(),
            disbursed_receivable_due_account_id: CalaAccountId::new(),
            disbursed_receivable_overdue_account_id: CalaAccountId::new(),
            disbursed_defaulted_account_id: CalaAccountId::new(),
            collateral_account_id: CalaAccountId::new(),
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
pub struct CreditFacilityProposalAccountIds {
    pub facility_account_id: CalaAccountId,
    pub collateral_account_id: CalaAccountId,
}

impl CreditFacilityProposalAccountIds {
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
pub struct CreditFacilityCreation {
    pub entity_id: CreditFacilityId,
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub credit_facility_account_ids: CreditFacilityLedgerAccountIds,
    pub facility_amount: UsdCents,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityProposalCreation {
    pub entity_id: CreditFacilityProposalId,
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub credit_facility_proposal_account_ids: CreditFacilityProposalAccountIds,
    pub facility_amount: UsdCents,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityActivation {
    pub entity_id: CreditFacilityId,
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub credit_facility_account_ids: CreditFacilityLedgerAccountIds,
    pub debit_account_id: CalaAccountId,
    pub facility_amount: UsdCents,
    pub structuring_fee_amount: UsdCents,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityInterestAccrual {
    pub credit_facility_id: CreditFacilityId,
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub interest: UsdCents,
    pub period: InterestPeriod,
    pub credit_facility_account_ids: CreditFacilityLedgerAccountIds,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityInterestAccrualCycle {
    pub credit_facility_id: CreditFacilityId,
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub interest: UsdCents,
    pub effective: chrono::NaiveDate,
    pub credit_facility_account_ids: CreditFacilityLedgerAccountIds,
}
