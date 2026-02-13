mod entity;
pub mod error;

use core_credit_collection::PaymentId;

use crate::primitives::LedgerTxId;

pub use entity::{Liquidation, LiquidationEvent, NewLiquidation};
pub(super) use error::LiquidationError;

#[derive(Clone, Debug)]
pub struct RecordProceedsFromLiquidationData {
    pub ledger_tx_id: LedgerTxId,
    pub payment_id: PaymentId,
}

impl RecordProceedsFromLiquidationData {
    pub(crate) fn new(ledger_tx_id: LedgerTxId, payment_id: PaymentId) -> Self {
        Self {
            payment_id,
            ledger_tx_id,
        }
    }
}
