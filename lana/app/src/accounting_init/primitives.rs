pub use cala_ledger::primitives::JournalId as LedgerJournalId;

use chart_of_accounts::ChartId;

use crate::{credit_facility::CreditFacilityAccountFactories, deposit::DepositAccountFactories};

#[derive(Clone, Copy)]
pub struct ChartIds {
    pub primary: ChartId,
    pub off_balance_sheet: ChartId,
}

#[derive(Clone)]
pub struct DepositsSeed {
    pub factories: DepositAccountFactories,
}

#[derive(Clone)]
pub struct CreditFacilitiesSeed {
    pub factories: CreditFacilityAccountFactories,
}
