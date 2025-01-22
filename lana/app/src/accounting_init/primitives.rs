pub use cala_ledger::primitives::JournalId as LedgerJournalId;

use chart_of_accounts::{ChartId, ControlSubAccountDetails};
use statements::TrialBalanceStatementId;

#[derive(Clone, Copy)]
pub struct ChartIds {
    pub primary: ChartId,
    pub off_balance_sheet: ChartId,
}

#[derive(Clone, Copy)]
pub struct TrialBalanceStatementIds {
    pub primary: TrialBalanceStatementId,
    pub off_balance_sheet: TrialBalanceStatementId,
}

#[derive(Clone)]
pub struct DepositsAccountPaths {
    pub deposits: ControlSubAccountDetails,
}

#[derive(Clone)]
pub struct CreditFacilitiesAccountPaths {
    pub collateral: ControlSubAccountDetails,
    pub facility: ControlSubAccountDetails,
    pub disbursed_receivable: ControlSubAccountDetails,
    pub interest_receivable: ControlSubAccountDetails,
    pub interest_income: ControlSubAccountDetails,
    pub fee_income: ControlSubAccountDetails,
}
