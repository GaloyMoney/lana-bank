mod entity;
pub mod error;

use cala_ledger::AccountId as CalaAccountId;
use core_money::{Satoshis, UsdCents};

pub use entity::{Liquidation, LiquidationEvent};
pub use error::LiquidationError;

use crate::ledger::FacilityProceedsFromLiquidationAccountId;

#[cfg(feature = "json-schema")]
pub use entity::LiquidationEvent;

#[derive(Clone, Debug)]
pub struct RecordProceedsFromLiquidationData {
    pub liquidation_proceeds_omnibus_account_id: CalaAccountId,
    pub proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,
    pub amount_received: UsdCents,
    pub collateral_in_liquidation_account_id: CalaAccountId,
    pub liquidated_collateral_account_id: CalaAccountId,
    pub amount_liquidated: Satoshis,
}
