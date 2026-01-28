pub mod constants;
mod seed;

pub mod error;

use tracing_macros::record_error_severity;

use crate::{
    accounting::Accounting, app::AccountingInitConfig, balance_sheet::BalanceSheets,
    credit::Credit, deposit::Deposits, primitives::CalaJournalId,
    profit_and_loss::ProfitAndLossStatements, trial_balance::TrialBalances,
};

use cala_ledger::CalaLedger;

use error::*;

#[derive(Clone)]
pub struct JournalInit {
    pub journal_id: CalaJournalId,
}

impl JournalInit {
    #[record_error_severity]
    #[tracing::instrument(name = "accounting_init.journal", skip_all)]
    pub async fn journal(cala: &CalaLedger) -> Result<Self, AccountingInitError> {
        seed::journal::init(cala).await
    }
}

#[derive(Clone)]
pub struct StatementsInit;

impl StatementsInit {
    #[record_error_severity]
    #[tracing::instrument(name = "accounting_init.statements", skip_all)]
    pub async fn statements(accounting: &Accounting) -> Result<(), AccountingInitError> {
        seed::statements::init(
            accounting.trial_balances(),
            accounting.profit_and_loss(),
            accounting.balance_sheets(),
        )
        .await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct ChartsInit;

impl ChartsInit {
    #[record_error_severity]
    #[tracing::instrument(name = "accounting_init.charts_of_accounts", skip_all)]
    pub async fn charts_of_accounts(
        accounting: &Accounting,
        credit: &Credit,
        deposit: &Deposits,
        accounting_init_config: AccountingInitConfig,
    ) -> Result<(), AccountingInitError> {
        seed::charts_of_accounts::init(accounting, credit, deposit, accounting_init_config).await
    }
}
