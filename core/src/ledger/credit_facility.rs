use serde::{Deserialize, Serialize};

use crate::primitives::{LedgerAccountId, LedgerTxId, UsdCents};

use super::CustomerLedgerAccountIds;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CreditFacilityAccountIds {
    pub account_id: LedgerAccountId,
}

impl CreditFacilityAccountIds {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            account_id: LedgerAccountId::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreditFacilityApprovalData {
    pub facility: UsdCents,
    pub tx_ref: String,
    pub tx_id: LedgerTxId,
    pub credit_facility_account_ids: CreditFacilityAccountIds,
    pub customer_account_ids: CustomerLedgerAccountIds,
}
