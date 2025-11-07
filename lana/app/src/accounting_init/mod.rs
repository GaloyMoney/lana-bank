pub mod constants;
mod seed;

pub mod error;

use crate::{
    accounting::{Accounting, ChartOfAccounts},
    app::AccountingInitConfig,
    balance_sheet::BalanceSheets,
    credit::Credit,
    deposit::Deposits,
    fiscal_year::FiscalYears,
    primitives::CalaJournalId,
    profit_and_loss::ProfitAndLossStatements,
    trial_balance::TrialBalances,
};

use cala_ledger::CalaLedger;
use chrono::{NaiveDate, Utc};
use error::*;

#[derive(Clone)]
pub struct JournalInit {
    pub journal_id: CalaJournalId,
}

impl JournalInit {
    #[tracing::instrument(name = "accounting_init.journal", skip_all, err)]
    pub async fn journal(cala: &CalaLedger) -> Result<Self, AccountingInitError> {
        seed::journal::init(cala).await
    }
}

#[derive(Clone)]
pub struct StatementsInit;

impl StatementsInit {
    #[tracing::instrument(name = "accounting_init.statements", skip_all, err)]
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
    #[tracing::instrument(name = "accounting_init.charts_of_accounts", skip_all, err)]
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

pub struct FiscalYearInit;

impl FiscalYearInit {
    pub async fn init_first_fiscal_year(
        accounting: &Accounting,
        chart_opening_date: Option<NaiveDate>,
    ) -> Result<(), AccountingInitError> {
        let chart = accounting.chart_of_accounts().find_by_id(chart_id).await?;

        // TODO: Can this config be optional?
        let opened_as_of = chart_opening_date.unwrap_or_else(|| Utc::now().date_naive());

        let chart_id = accounting
            .chart_of_accounts()
            .find_by_reference(crate::accounting_init::constants::CHART_REF)
            .await?
            .id;
        Ok(accounting
            .fiscal_year()
            .init_first_fiscal_year(opened_as_of, chart_id, chart.account_set_id)
            .await?)
    }
}
