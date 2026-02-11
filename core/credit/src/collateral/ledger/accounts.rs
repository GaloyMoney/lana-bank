use serde::{Deserialize, Serialize};

use crate::{ledger::FacilityProceedsFromLiquidationAccountId, primitives::CalaAccountId};

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

impl CollateralLedgerAccountIds {
    pub(crate) fn new() -> Self {
        Self {
            collateral_account_id: CalaAccountId::new(),
            liquidated_collateral_account_id: CalaAccountId::new(),
            collateral_in_liquidation_account_id: CalaAccountId::new(),
        }
    }
}

/// Account IDs needed for recording proceeds from liquidation.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct LiquidationProceedsAccountIds {
    pub liquidation_proceeds_omnibus_account_id: CalaAccountId,
    pub proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,
    pub collateral_in_liquidation_account_id: CalaAccountId,
    pub liquidated_collateral_account_id: CalaAccountId,
}

impl LiquidationProceedsAccountIds {
    pub fn new(
        collateral_accounts: &CollateralLedgerAccountIds,
        facility_proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,
        liquidation_proceeds_omnibus_account_id: CalaAccountId,
    ) -> Self {
        Self {
            liquidation_proceeds_omnibus_account_id,
            proceeds_from_liquidation_account_id: facility_proceeds_from_liquidation_account_id,
            collateral_in_liquidation_account_id: collateral_accounts
                .collateral_in_liquidation_account_id,
            liquidated_collateral_account_id: collateral_accounts.liquidated_collateral_account_id,
        }
    }
}
