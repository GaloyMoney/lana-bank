pub use cala_ledger::primitives::JournalId as LedgerJournalId;

use chart_of_accounts::{ChartId, ControlSubAccountDetails};

pub use crate::primitives::{ProfitAndLossStatementId, TrialBalanceId};

#[derive(Clone, Copy)]
pub struct ChartIds {
    pub primary: ChartId,
    pub off_balance_sheet: ChartId,
}

#[derive(Clone)]
pub struct DepositsAccountPaths {
    pub deposits: ControlSubAccountDetails,
    pub deposits_omnibus: ControlSubAccountDetails,
}

#[derive(Clone)]
pub struct CreditFacilitiesAccountPaths {
    pub collateral: ControlSubAccountDetails,
    pub collateral_omnibus: ControlSubAccountDetails,
    pub facility: ControlSubAccountDetails,
    pub facility_omnibus: ControlSubAccountDetails,
    pub disbursed_receivable: ControlSubAccountDetails,
    pub interest_receivable: ControlSubAccountDetails,
    pub interest_income: ControlSubAccountDetails,
    pub fee_income: ControlSubAccountDetails,
}
