mod entity;
pub(crate) mod error;

use cala_ledger::AccountId as CalaAccountId;
use money::{Satoshis, UsdCents};

use crate::{
    collateral::ledger::LiquidationProceedsAccountIds,
    ledger::FacilityProceedsFromLiquidationAccountId, primitives::LedgerTxId,
};

pub use entity::{Liquidation, LiquidationEvent, NewLiquidation};
pub(super) use error::LiquidationError;

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

impl RecordProceedsFromLiquidationData {
    pub(crate) fn new(
        account_ids: LiquidationProceedsAccountIds,
        amount_received: UsdCents,
        amount_liquidated: Satoshis,
        ledger_tx_id: LedgerTxId,
    ) -> Self {
        Self {
            liquidation_proceeds_omnibus_account_id: account_ids
                .liquidation_proceeds_omnibus_account_id,
            proceeds_from_liquidation_account_id: account_ids.proceeds_from_liquidation_account_id,
            collateral_in_liquidation_account_id: account_ids.collateral_in_liquidation_account_id,
            liquidated_collateral_account_id: account_ids.liquidated_collateral_account_id,
            amount_received,
            amount_liquidated,
            ledger_tx_id,
        }
    }
}
