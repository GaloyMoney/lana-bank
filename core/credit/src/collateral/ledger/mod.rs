mod accounts;
mod error;
mod trait_def;

pub use trait_def::CollateralLedgerOps;

pub use accounts::{
    CollateralLedgerAccountIds, FacilityLedgerAccountIdsForLiquidation,
    LiquidationProceedsAccountIds,
};
pub use error::CollateralLedgerError;

use crate::ledger::InternalAccountSetDetails;

#[derive(Clone, Copy)]
pub struct CollateralAccountSets {
    pub collateral: InternalAccountSetDetails,
    pub collateral_in_liquidation: InternalAccountSetDetails,
    pub liquidated_collateral: InternalAccountSetDetails,
}
