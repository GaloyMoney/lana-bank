use serde::{Deserialize, Serialize};

use crate::{FacilityProceedsFromLiquidationAccountId, primitives::CalaAccountId};

use super::PendingCreditFacilityAccountIds;

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

impl CollateralLedgerAccountIds {
    pub(crate) fn from_pending_credit_facility_account_ids(
        pending_ids: PendingCreditFacilityAccountIds,
        liquidation_proceeds_omnibus_account_id: CalaAccountId,
    ) -> Self {
        Self {
            collateral_account_id: pending_ids.collateral_account_id,
            liquidated_collateral_account_id: CalaAccountId::new(),
            collateral_in_liquidation_account_id: CalaAccountId::new(),
            liquidation_proceeds_omnibus_account_id,
            facility_proceeds_from_liquidation_account_id: pending_ids
                .facility_proceeds_from_liquidation_account_id,
            facility_uncovered_outstanding_account_id: pending_ids
                .facility_uncovered_outstanding_account_id,
            facility_payment_holding_account_id: pending_ids.facility_payment_holding_account_id,
        }
    }
}
