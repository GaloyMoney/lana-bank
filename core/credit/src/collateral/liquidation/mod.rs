mod entity;
pub mod error;

use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as CalaAccountId;
use money::{Satoshis, UsdCents};

use crate::{ledger::FacilityProceedsFromLiquidationAccountId, primitives::LedgerTxId};

pub use entity::{Liquidation, LiquidationEvent, NewLiquidation};
pub use error::LiquidationError;

#[derive(Clone, Debug)]
pub struct RecordProceedsFromLiquidationData {
    pub liquidation_proceeds_omnibus_account_id: CalaAccountId,
    pub proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,
    pub collateral_in_liquidation_account_id: CalaAccountId,
    pub liquidated_collateral_account_id: CalaAccountId,

    pub amount_received: UsdCents,
    pub amount_liquidated: Satoshis,

    pub ledger_tx_id: LedgerTxId,
}

/// Account IDs needed for recording proceeds from liquidation.
/// These come from the credit facility and collateral accounts.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct LiquidationProceedsAccountIds {
    pub liquidation_proceeds_omnibus_account_id: CalaAccountId,
    pub proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,
    pub collateral_in_liquidation_account_id: CalaAccountId,
    pub liquidated_collateral_account_id: CalaAccountId,
}
