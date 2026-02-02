use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

// Re-export types from dependencies
pub use cala_ledger::primitives::{
    AccountId as CalaAccountId, AccountSetId as CalaAccountSetId, Currency,
    DebitOrCredit as LedgerDebitOrCredit, JournalId as LedgerJournalId,
    TransactionId as LedgerTxId, TxTemplateId as LedgerTxTemplateId,
};
pub use core_money::*;

// Re-export EffectiveDate from terms crate
pub use core_credit_terms::EffectiveDate;

// Collections-specific entity IDs
es_entity::entity_id! {
    FacilityId,
    PaymentId,
    PaymentAllocationId,
    ObligationId;

    FacilityId => job::JobId,
    ObligationId => job::JobId,

    PaymentId => LedgerTxId,
    PaymentAllocationId => LedgerTxId,
}

// Obligation-related enums
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObligationStatus {
    NotYetDue,
    Due,
    Overdue,
    Defaulted,
    Paid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum ObligationType {
    Disbursal,
    Interest,
}

// Obligation amounts struct
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ObligationsAmounts {
    pub disbursed: UsdCents,
    pub interest: UsdCents,
}

impl std::ops::Add<ObligationsAmounts> for ObligationsAmounts {
    type Output = Self;

    fn add(self, other: ObligationsAmounts) -> Self {
        Self {
            disbursed: self.disbursed + other.disbursed,
            interest: self.interest + other.interest,
        }
    }
}

impl ObligationsAmounts {
    pub const ZERO: Self = Self {
        disbursed: UsdCents::ZERO,
        interest: UsdCents::ZERO,
    };

    pub fn total(&self) -> UsdCents {
        self.interest + self.disbursed
    }

    pub fn is_zero(&self) -> bool {
        self.disbursed.is_zero() && self.interest.is_zero()
    }
}

// Payment-related structs
#[derive(Debug, Clone, Copy)]
pub struct PaymentSourceAccountId(CalaAccountId);

impl From<PaymentSourceAccountId> for CalaAccountId {
    fn from(account_id: PaymentSourceAccountId) -> Self {
        account_id.0
    }
}

impl PaymentSourceAccountId {
    pub const fn new(account_id: CalaAccountId) -> Self {
        Self(account_id)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PaymentDetailsForAllocation {
    pub payment_id: PaymentId,
    pub amount: UsdCents,
    pub facility_id: FacilityId,
    pub facility_payment_holding_account_id: CalaAccountId,
    pub effective: chrono::NaiveDate,
}

// Obligation reallocation data structs
pub struct ObligationDueReallocationData {
    pub tx_id: LedgerTxId,
    pub amount: UsdCents,
    pub not_yet_due_account_id: CalaAccountId,
    pub due_account_id: CalaAccountId,
    pub effective: chrono::NaiveDate,
}

pub struct ObligationOverdueReallocationData {
    pub tx_id: LedgerTxId,
    pub amount: UsdCents,
    pub due_account_id: CalaAccountId,
    pub overdue_account_id: CalaAccountId,
    pub effective: chrono::NaiveDate,
}

pub struct ObligationDefaultedReallocationData {
    pub tx_id: LedgerTxId,
    pub amount: UsdCents,
    pub receivable_account_id: CalaAccountId,
    pub defaulted_account_id: CalaAccountId,
    pub effective: chrono::NaiveDate,
}

// Balance update related types
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum BalanceUpdatedSource {
    Obligation(ObligationId),
    PaymentAllocation(PaymentAllocationId),
}

impl From<ObligationId> for BalanceUpdatedSource {
    fn from(obligation_id: ObligationId) -> Self {
        Self::Obligation(obligation_id)
    }
}

impl From<PaymentAllocationId> for BalanceUpdatedSource {
    fn from(allocation_id: PaymentAllocationId) -> Self {
        Self::PaymentAllocation(allocation_id)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct BalanceUpdateData {
    pub source_id: BalanceUpdatedSource,
    pub ledger_tx_id: LedgerTxId,
    pub balance_type: ObligationType,
    pub amount: UsdCents,
    pub updated_at: DateTime<Utc>,
}

// Ledger account structs
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ObligationReceivableAccountIds {
    pub not_yet_due: CalaAccountId,
    pub due: CalaAccountId,
    pub overdue: CalaAccountId,
}

impl ObligationReceivableAccountIds {
    #[allow(clippy::new_without_default)]
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            not_yet_due: CalaAccountId::new(),
            due: CalaAccountId::new(),
            overdue: CalaAccountId::new(),
        }
    }

    pub fn id_for_status(&self, status: ObligationStatus) -> Option<CalaAccountId> {
        match status {
            ObligationStatus::NotYetDue => Some(self.not_yet_due),
            ObligationStatus::Due => Some(self.due),
            ObligationStatus::Overdue | ObligationStatus::Defaulted => Some(self.overdue),
            ObligationStatus::Paid => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PaymentLedgerAccountIds {
    pub facility_payment_holding_account_id: CalaAccountId,
    pub facility_uncovered_outstanding_account_id: CalaAccountId,
    pub payment_source_account_id: PaymentSourceAccountId,
}
