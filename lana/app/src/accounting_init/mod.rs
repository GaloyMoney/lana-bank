pub mod constants;
mod primitives;
mod seed;

pub mod error;

use chart_of_accounts::ChartId;

use crate::{
    balance_sheet::BalanceSheets, cash_flow::CashFlowStatements,
    chart_of_accounts::ChartOfAccounts, new_chart_of_accounts::NewChartOfAccounts,
    profit_and_loss::ProfitAndLossStatements, trial_balance::TrialBalances,
};

use cala_ledger::CalaLedger;
use error::*;
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
pub struct ChartsInit {
    pub chart_ids: ChartIds,
}

impl ChartsInit {
    pub async fn charts_of_accounts(
        chart_of_accounts: &ChartOfAccounts,
        new_chart_of_accounts: &NewChartOfAccounts,
    ) -> Result<Self, AccountingInitError> {
        seed::charts_of_accounts::init(chart_of_accounts, new_chart_of_accounts).await
    }
}
