pub use crate::primitives::LedgerJournalId;

use chart_of_accounts::ChartId;

#[derive(Clone, Copy)]
pub struct ChartIds {
    pub primary: ChartId,
    pub off_balance_sheet: ChartId,
}
