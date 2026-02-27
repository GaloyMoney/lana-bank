use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as CalaAccountId;
use core_credit_collection::PaymentSourceAccountId;

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct FacilityProceedsFromLiquidationAccountId(CalaAccountId);

impl FacilityProceedsFromLiquidationAccountId {
    pub fn new() -> Self {
        Self(CalaAccountId::new())
    }

    pub const fn into_inner(self) -> CalaAccountId {
        self.0
    }
}

impl Default for FacilityProceedsFromLiquidationAccountId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<FacilityProceedsFromLiquidationAccountId> for PaymentSourceAccountId {
    fn from(account: FacilityProceedsFromLiquidationAccountId) -> Self {
        Self::new(account.0)
    }
}

impl From<FacilityProceedsFromLiquidationAccountId> for CalaAccountId {
    fn from(account: FacilityProceedsFromLiquidationAccountId) -> Self {
        account.0
    }
}

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

impl Default for CollateralLedgerAccountIds {
    fn default() -> Self {
        Self::new()
    }
}

impl CollateralLedgerAccountIds {
    pub fn new() -> Self {
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

impl FacilityLedgerAccountIdsForLiquidation {
    pub fn new(
        proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,
        payment_holding_account_id: CalaAccountId,
        uncovered_outstanding_account_id: CalaAccountId,
    ) -> Self {
        Self {
            proceeds_from_liquidation_account_id,
            payment_holding_account_id,
            uncovered_outstanding_account_id,
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
