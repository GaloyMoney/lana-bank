pub mod constants;
mod primitives;
mod seed;

pub mod error;

use chart_of_accounts::ChartId;

use crate::{
    balance_sheet::BalanceSheets, chart_of_accounts::ChartOfAccounts,
    profit_and_loss::ProfitAndLossStatements, trial_balance::TrialBalances,
};

use cala_ledger::CalaLedger;

use error::*;
pub use primitives::CreditFacilitiesAccountPaths;
use primitives::*;

#[derive(Clone)]
pub struct JournalInit {
    pub journal_id: LedgerJournalId,
}

impl JournalInit {
    pub async fn journal(cala: &CalaLedger) -> Result<Self, AccountingInitError> {
        seed::journal::init(cala).await
    }
}

#[derive(Clone)]
pub struct StatementsInit;

impl StatementsInit {
    pub async fn statements(
        trial_balances: &TrialBalances,
        pl_statements: &ProfitAndLossStatements,
        balance_sheets: &BalanceSheets,
    ) -> Result<(), AccountingInitError> {
        seed::statements::init(trial_balances, pl_statements, balance_sheets).await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct ChartsInit {
    pub chart_ids: ChartIds,
    pub deposits: DepositsAccountPaths,
    pub credit_facilities: CreditFacilitiesAccountPaths,
}

impl ChartsInit {
    pub async fn charts_of_accounts(
        trial_balances: &TrialBalances,
        pl_statements: &ProfitAndLossStatements,
        chart_of_accounts: &ChartOfAccounts,
    ) -> Result<Self, AccountingInitError> {
        seed::charts_of_accounts::init(trial_balances, pl_statements, chart_of_accounts).await
    }
}
