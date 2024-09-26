use crate::primitives::{LedgerTxId, UsdCents};

use super::{CreditFacilityAccountIds, CustomerLedgerAccountIds};

#[derive(Debug, Clone)]
pub struct DisbursementData {
    pub amount: UsdCents,
    pub tx_ref: String,
    pub tx_id: LedgerTxId,
    pub account_ids: CreditFacilityAccountIds,
    pub customer_account_ids: CustomerLedgerAccountIds,
}
