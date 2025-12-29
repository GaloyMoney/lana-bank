use crate::primitives::*;
#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ObligationAccounts {
    pub receivable_account_id: CalaAccountId,
}

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
