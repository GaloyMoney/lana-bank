#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::primitives::{CalaAccountId, LedgerTxId, UsdCents};

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
pub struct PendingCreditFacilityCreation {
    pub tx_id: LedgerTxId,
    pub tx_ref: String,
    pub pending_credit_facility_account_ids: PendingCreditFacilityAccountIds,
    pub facility_amount: UsdCents,
}
