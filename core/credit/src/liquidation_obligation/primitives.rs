use crate::primitives::*;

pub struct LiquidationObligationDefaultedReallocationData {
    pub tx_id: LedgerTxId,
    pub amount: UsdCents,
    pub receivable_account_id: CalaAccountId,
    pub defaulted_account_id: CalaAccountId,
    pub effective: chrono::NaiveDate,
}
