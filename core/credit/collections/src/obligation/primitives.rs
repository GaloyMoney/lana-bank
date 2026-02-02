use crate::primitives::*;
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
