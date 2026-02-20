#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cala_ledger_core_types::primitives::AccountId as CalaAccountId;
use core_credit_collection::PaymentSourceAccountId;

use crate::{
    FacilityDurationType, InterestPeriod,
    primitives::{CreditFacilityId, CustomerType, DisbursalId, LedgerTxId, UsdCents},
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CreditFacilityLedgerAccountIds {
    pub facility_account_id: CalaAccountId,
    pub disbursed_receivable_not_yet_due_account_id: CalaAccountId,
    pub disbursed_receivable_due_account_id: CalaAccountId,
    pub disbursed_receivable_overdue_account_id: CalaAccountId,
    pub disbursed_defaulted_account_id: CalaAccountId,

    /// Holds funds received from liquidation.
    pub proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,

    pub interest_receivable_not_yet_due_account_id: CalaAccountId,
    pub interest_receivable_due_account_id: CalaAccountId,
    pub interest_receivable_overdue_account_id: CalaAccountId,
    pub interest_defaulted_account_id: CalaAccountId,
    pub interest_income_account_id: CalaAccountId,
    pub fee_income_account_id: CalaAccountId,

    /// Holds funds meant for payment allocation.
    pub payment_holding_account_id: CalaAccountId,

    /// Holds outstanding not yet covered by an unallocated payment.
    pub uncovered_outstanding_account_id: CalaAccountId,
}

impl CreditFacilityLedgerAccountIds {
    #[allow(clippy::new_without_default)]
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            facility_account_id: CalaAccountId::new(),
            disbursed_receivable_not_yet_due_account_id: CalaAccountId::new(),
            disbursed_receivable_due_account_id: CalaAccountId::new(),
            disbursed_receivable_overdue_account_id: CalaAccountId::new(),
            disbursed_defaulted_account_id: CalaAccountId::new(),
            proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId::new(),
            interest_receivable_not_yet_due_account_id: CalaAccountId::new(),
            interest_receivable_due_account_id: CalaAccountId::new(),
            interest_receivable_overdue_account_id: CalaAccountId::new(),
            interest_defaulted_account_id: CalaAccountId::new(),
            interest_income_account_id: CalaAccountId::new(),
            fee_income_account_id: CalaAccountId::new(),
            uncovered_outstanding_account_id: CalaAccountId::new(),
            payment_holding_account_id: CalaAccountId::new(),
        }
    }
}

impl From<PendingCreditFacilityAccountIds> for CreditFacilityLedgerAccountIds {
    fn from(
        PendingCreditFacilityAccountIds {
            facility_account_id,
            facility_proceeds_from_liquidation_account_id: proceeds_from_liquidation_account_id,
            facility_uncovered_outstanding_account_id: uncovered_outstanding_account_id,
            facility_payment_holding_account_id: payment_holding_account_id,
        }: PendingCreditFacilityAccountIds,
    ) -> Self {
        Self {
            facility_account_id,
            proceeds_from_liquidation_account_id,
            uncovered_outstanding_account_id,
            payment_holding_account_id,

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
    pub facility_proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,
    pub facility_uncovered_outstanding_account_id: CalaAccountId,
    pub facility_payment_holding_account_id: CalaAccountId,
}

impl PendingCreditFacilityAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            facility_account_id: CalaAccountId::new(),
            facility_proceeds_from_liquidation_account_id:
                FacilityProceedsFromLiquidationAccountId::new(),
            facility_uncovered_outstanding_account_id: CalaAccountId::new(),
            facility_payment_holding_account_id: CalaAccountId::new(),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct InterestAccrualCycleLedgerAccountIds {
    receivable_not_yet_due_account_id: CalaAccountId,
    receivable_due_account_id: CalaAccountId,
    receivable_overdue_account_id: CalaAccountId,
    defaulted_account_id: CalaAccountId,
    interest_income_account_id: CalaAccountId,
    uncovered_outstanding_account_id: CalaAccountId,
}

impl From<CreditFacilityLedgerAccountIds> for InterestAccrualCycleLedgerAccountIds {
    fn from(credit_facility_account_ids: CreditFacilityLedgerAccountIds) -> Self {
        Self {
            receivable_not_yet_due_account_id: credit_facility_account_ids
                .interest_receivable_not_yet_due_account_id,
            receivable_due_account_id: credit_facility_account_ids
                .interest_receivable_due_account_id,
            receivable_overdue_account_id: credit_facility_account_ids
                .interest_receivable_overdue_account_id,
            defaulted_account_id: credit_facility_account_ids.interest_defaulted_account_id,
            interest_income_account_id: credit_facility_account_ids.interest_income_account_id,
            uncovered_outstanding_account_id: credit_facility_account_ids
                .uncovered_outstanding_account_id,
        }
    }
}

impl InterestAccrualCycleLedgerAccountIds {
    pub fn defaulted_account_id(&self) -> CalaAccountId {
        self.defaulted_account_id
    }
}

impl From<InterestAccrualCycleLedgerAccountIds>
    for core_credit_collection::ObligationReceivableAccountIds
{
    fn from(account_ids: InterestAccrualCycleLedgerAccountIds) -> Self {
        Self {
            not_yet_due: account_ids.receivable_not_yet_due_account_id,
            due: account_ids.receivable_due_account_id,
            overdue: account_ids.receivable_overdue_account_id,
        }
    }
}

impl From<InterestAccrualCycleLedgerAccountIds> for InterestPostingAccountIds {
    fn from(account_ids: InterestAccrualCycleLedgerAccountIds) -> Self {
        Self {
            receivable_not_yet_due: account_ids.receivable_not_yet_due_account_id,
            income: account_ids.interest_income_account_id,
            uncovered_outstanding: account_ids.uncovered_outstanding_account_id,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InterestPostingAccountIds {
    pub receivable_not_yet_due: CalaAccountId,
    pub income: CalaAccountId,
    pub uncovered_outstanding: CalaAccountId,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityCompletion {
    pub tx_id: LedgerTxId,
}

#[derive(Debug, Clone)]
pub struct PendingCreditFacilityCreation {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub pending_credit_facility_account_ids: PendingCreditFacilityAccountIds,
    pub facility_amount: UsdCents,
}

pub struct InitialDisbursalOnActivation {
    pub id: DisbursalId,
    pub amount: UsdCents,
}

pub struct StructuringFeeOnActivation {
    pub tx_id: LedgerTxId,
    pub amount: UsdCents,
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
    pub initial_disbursal: Option<InitialDisbursalOnActivation>,
    pub structuring_fee: Option<StructuringFeeOnActivation>,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityInterestAccrual {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub interest: UsdCents,
    pub period: InterestPeriod,
    pub account_ids: InterestAccrualCycleLedgerAccountIds,
}

#[derive(Debug, Clone)]
pub struct CreditFacilityInterestAccrualCycle {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub interest: UsdCents,
    pub effective: chrono::NaiveDate,
    pub account_ids: InterestAccrualCycleLedgerAccountIds,
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct FacilityProceedsFromLiquidationAccountId(CalaAccountId);

impl FacilityProceedsFromLiquidationAccountId {
    pub fn new() -> Self {
        Self(CalaAccountId::new())
    }

    pub const fn into_inner(self) -> CalaAccountId {
        self.0
    }
}

impl Default for FacilityProceedsFromLiquidationAccountId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<FacilityProceedsFromLiquidationAccountId> for PaymentSourceAccountId {
    fn from(account: FacilityProceedsFromLiquidationAccountId) -> Self {
        Self::new(account.0)
    }
}

impl From<FacilityProceedsFromLiquidationAccountId> for CalaAccountId {
    fn from(account: FacilityProceedsFromLiquidationAccountId) -> Self {
        account.0
    }
}
