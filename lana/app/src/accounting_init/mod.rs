pub mod constants;
mod seed;

pub mod error;

use crate::{
    accounting::{Accounting, ChartOfAccounts},
    app::AccountingInitConfig,
    balance_sheet::BalanceSheets,
    credit::Credit,
    deposit::Deposits,
    primitives::CalaJournalId,
    profit_and_loss::ProfitAndLossStatements,
    trial_balance::TrialBalances,
};

use cala_ledger::CalaLedger;
use error::*;

#[derive(Clone)]
pub struct JournalInit {
    pub journal_id: CalaJournalId,
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
pub struct ChartsInit;

impl ChartsInit {
    pub async fn charts_of_accounts(
        accounting: &Accounting,
        credit: &Credit,
        deposit: &Deposits,
        accounting_init_config: AccountingInitConfig,
    ) -> Result<(), AccountingInitError> {
        seed::charts_of_accounts::init(
            accounting.chart_of_accounts(),
            accounting.trial_balances(),
            credit,
            deposit,
            accounting.balance_sheets(),
            accounting.profit_and_loss(),
            accounting_init_config,
        )
        .await
    }
}
