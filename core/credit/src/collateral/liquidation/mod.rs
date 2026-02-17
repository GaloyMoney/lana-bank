mod entity;
pub mod error;

use crate::primitives::{LedgerTxId, PaymentId};

pub use entity::{Liquidation, LiquidationEvent, NewLiquidation};
pub(super) use error::LiquidationError;

#[derive(Clone, Debug)]
pub(super) struct RecordProceedsFromLiquidationData {
    pub ledger_tx_id: LedgerTxId,
    pub payment_id: PaymentId,
}

impl RecordProceedsFromLiquidationData {
    pub(super) fn new(ledger_tx_id: LedgerTxId, payment_id: PaymentId) -> Self {
        Self {
            payment_id,
            ledger_tx_id,
        }
    }
}
