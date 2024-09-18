use crate::primitives::{LedgerTxId, UsdCents};

use super::{CustomerLedgerAccountIds, LoanAccountIds};

#[derive(Debug, Clone)]
pub struct DisbursementData {
    pub amount: UsdCents,
    pub tx_ref: String,
    pub tx_id: LedgerTxId,
    pub account_ids: LoanAccountIds,
    pub customer_account_ids: CustomerLedgerAccountIds,
}
