use serde::{Deserialize, Serialize};

use crate::primitives::CalaAccountId;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct CollateralLedgerAccountIds {
    /// Holds BTC collateral for this credit facility.
    pub collateral_account_id: CalaAccountId,

    /// Holds BTC collateral for this credit facility, that has
    /// already been liquidated.
    pub liquidated_collateral_account_id: CalaAccountId,

    /// Holds BTC collateral for this credit facility, that is being
    /// liquidated.
    pub collateral_in_liquidation_account_id: CalaAccountId,
}
