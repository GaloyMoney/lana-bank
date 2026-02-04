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

    pub(crate) liquidation_proceeds_omnibus_account_id: CalaAccountId,

    /// Holds proceeds received from liquidator for the connected
    /// facility.
    pub(crate) facility_proceeds_from_liquidation_account_id:
        FacilityProceedsFromLiquidationAccountId,

    pub(crate) facility_uncovered_outstanding_account_id: CalaAccountId,

    pub(crate) facility_payment_holding_account_id: CalaAccountId,
}
