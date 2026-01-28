mod error;

use tracing::instrument;
use tracing_macros::record_error_severity;

use cala_ledger::{CalaLedger, JournalId};
use es_entity::clock::ClockHandle;

pub use error::LiquidationLedgerError;

#[derive(Clone)]
pub struct LiquidationLedger {
    _cala: CalaLedger,
    _clock: ClockHandle,
    _journal_id: JournalId,
}

impl LiquidationLedger {
    #[record_error_severity]
    #[instrument(name = "core_credit.liquidation.ledger.init", skip_all)]
    pub async fn init(
        cala: &CalaLedger,
        journal_id: JournalId,
        clock: ClockHandle,
    ) -> Result<Self, LiquidationLedgerError> {
        Ok(Self {
            _cala: cala.clone(),
            _clock: clock,
            _journal_id: journal_id,
        })
    }
}
