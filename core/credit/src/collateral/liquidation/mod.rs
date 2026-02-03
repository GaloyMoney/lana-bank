mod entity;
pub mod error;

use serde::{Deserialize, Serialize};

use cala_ledger::AccountId as CalaAccountId;
use core_credit_collection::PaymentSourceAccountId;
use core_money::{Satoshis, UsdCents};

use crate::primitives::LedgerTxId;

pub use entity::NewLiquidationBuilder;
pub use entity::{Liquidation, LiquidationEvent, NewLiquidation};
pub use error::LiquidationError;

#[derive(Clone, Debug)]
pub struct RecordProceedsFromLiquidationData {
    pub liquidation_proceeds_omnibus_account_id: CalaAccountId,
    pub proceeds_from_liquidation_account_id: FacilityProceedsFromLiquidationAccountId,
    pub amount_received: UsdCents,
    pub collateral_in_liquidation_account_id: CalaAccountId,
    pub liquidated_collateral_account_id: CalaAccountId,
    pub amount_liquidated: Satoshis,
    pub ledger_tx_id: LedgerTxId,
}

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

impl From<&FacilityProceedsFromLiquidationAccountId> for PaymentSourceAccountId {
    fn from(account: &FacilityProceedsFromLiquidationAccountId) -> Self {
        Self::new(account.0)
    }
}

impl From<FacilityProceedsFromLiquidationAccountId> for CalaAccountId {
    fn from(account: FacilityProceedsFromLiquidationAccountId) -> Self {
        account.0
    }
}
