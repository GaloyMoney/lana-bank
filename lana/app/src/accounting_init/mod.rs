pub mod constants;
mod seed;

pub mod error;

use crate::{
    accounting::{Accounting, Chart, ChartOfAccounts},
    accounting_period::AccountingPeriods,
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
    ) -> Result<Chart, AccountingInitError> {
        seed::charts_of_accounts::init(
            accounting.chart_of_accounts(),
            accounting.trial_balances(),
            credit,
            deposit,
            accounting.balance_sheets(),
            accounting.profit_and_loss(),
            accounting.accounting_periods(),
            accounting_init_config,
        )
        .await
    }
}

pub struct AccountingPeriodsInit;

impl AccountingPeriodsInit {
    pub async fn open_initial_accounting_periods(
        accounting: &Accounting,
        chart: &Chart,
    ) -> Result<(), AccountingInitError> {
        let config = accounting
            .accounting_periods()
            .get_chart_of_accounts_integration_config(chart)
            .await?
            .unwrap();

        Ok(accounting
            .accounting_periods()
            .open_initial_periods(
                chart.id,
                chart.account_set_id,
                es_entity::prelude::sim_time::now().date_naive(),
                config.accounting_periods,
            )
            .await?)
    }
}
