use serde::{Deserialize, Serialize};

use crate::{
    ledger::{FacilityProceedsFromLiquidationAccountId, PendingCreditFacilityAccountIds},
    primitives::CalaAccountId,
};

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

/// Facility-level account IDs needed by the Collateral entity for liquidation operations.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct FacilityLedgerAccountIdsForLiquidation {
    pub proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,
    pub payment_holding_account_id: CalaAccountId,
    pub uncovered_outstanding_account_id: CalaAccountId,
}

impl From<PendingCreditFacilityAccountIds> for FacilityLedgerAccountIdsForLiquidation {
    fn from(ids: PendingCreditFacilityAccountIds) -> Self {
        Self {
            proceeds_from_liquidation_account_id: ids.facility_proceeds_from_liquidation_account_id,
            payment_holding_account_id: ids.facility_payment_holding_account_id,
            uncovered_outstanding_account_id: ids.facility_uncovered_outstanding_account_id,
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
        facility_account_ids: &FacilityLedgerAccountIdsForLiquidation,
        liquidation_proceeds_omnibus_account_id: CalaAccountId,
    ) -> Self {
        Self {
            liquidation_proceeds_omnibus_account_id,
            proceeds_from_liquidation_account_id: facility_account_ids
                .proceeds_from_liquidation_account_id,
            collateral_in_liquidation_account_id: collateral_accounts
                .collateral_in_liquidation_account_id,
            liquidated_collateral_account_id: collateral_accounts.liquidated_collateral_account_id,
        }
    }
}
