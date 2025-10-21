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
use chrono::{Datelike, Duration};
use core_accounting::accounting_period::{
    Period, chart_of_accounts_integration::AccountingPeriodBasis,
};
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

        let mut periods = vec![];

        for period_config in config.accounting_periods {
            let period = match period_config.basis {
                AccountingPeriodBasis::Month => Period::monthly_by_day_in_month(
                    period_config.first_period_start.day() as u8,
                    period_config.first_period_start,
                    Duration::days(period_config.grace_period_days.into()),
                ),
                AccountingPeriodBasis::Year => Period::annually_by_date(
                    period_config.first_period_start.day() as u8,
                    period_config.first_period_start.month() as u8,
                    period_config.first_period_start,
                    Duration::days(period_config.grace_period_days.into()),
                ),
            };
            periods.push(period.unwrap());
        }

        Ok(accounting
            .accounting_periods()
            .open_initial_periods(chart.id, chart.account_set_id, periods)
            .await?)
    }
}
