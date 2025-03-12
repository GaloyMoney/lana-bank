pub mod constants;
mod seed;

pub mod error;

use crate::{
    balance_sheet::BalanceSheets, cash_flow::CashFlowStatements,
    new_chart_of_accounts::NewChartOfAccounts, primitives::LedgerJournalId,
    profit_and_loss::ProfitAndLossStatements, trial_balance::TrialBalances,
};

use cala_ledger::CalaLedger;
use error::*;

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
        cash_flow_statements: &CashFlowStatements,
    ) -> Result<(), AccountingInitError> {
        seed::statements::init(
            trial_balances,
            pl_statements,
            balance_sheets,
            cash_flow_statements,
        )
        .await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct ChartsInit;

impl ChartsInit {
    pub async fn charts_of_accounts(
        new_chart_of_accounts: &NewChartOfAccounts,
    ) -> Result<(), AccountingInitError> {
        seed::charts_of_accounts::init(new_chart_of_accounts).await
    }
}
