use std::path::PathBuf;

use crate::{
    accounting::ChartId,
    accounting_init::{constants::*, *},
};

use rbac_types::Subject;

use super::module_config::{
    balance_sheet::*, credit::*, deposit::*, period::*, profit_and_loss::*,
};

pub(crate) async fn init(
    chart_of_accounts: &ChartOfAccounts,
    trial_balances: &TrialBalances,
    credit: &Credit,
    deposit: &Deposits,
    balance_sheet: &BalanceSheets,
    profit_and_loss: &ProfitAndLossStatements,
    accounting_periods: &AccountingPeriods,
    accounting_init_config: AccountingInitConfig,
) -> Result<Chart, AccountingInitError> {
    let AccountingInitConfig {
        chart_of_accounts_opening_date,
        chart_of_accounts_seed_path,
        ..
    } = accounting_init_config.clone();
    let opening_date = chart_of_accounts_opening_date.ok_or_else(|| {
        AccountingInitError::MissingConfig("chart_of_accounts_opening_date".to_string())
    })?;

    let chart = create_chart_of_accounts(chart_of_accounts, opening_date).await?;

    if let Some(path) = chart_of_accounts_seed_path {
        seed_chart_of_accounts(
            chart_of_accounts,
            trial_balances,
            credit,
            deposit,
            balance_sheet,
            profit_and_loss,
            accounting_periods,
            chart.id,
            path,
            accounting_init_config,
        )
        .await?;
    }
    Ok(chart)
}

async fn create_chart_of_accounts(
    chart_of_accounts: &ChartOfAccounts,
    opening_date: chrono::NaiveDate,
) -> Result<Chart, AccountingInitError> {
    if let Some(chart) = chart_of_accounts.find_by_reference(CHART_REF).await? {
        Ok(chart)
    } else {
        Ok(chart_of_accounts
            .create_chart(
                &Subject::System,
                CHART_NAME.to_string(),
                CHART_REF.to_string(),
                opening_date,
            )
            .await?)
    }
}

async fn seed_chart_of_accounts(
    chart_of_accounts: &ChartOfAccounts,
    trial_balances: &TrialBalances,
    credit: &Credit,
    deposit: &Deposits,
    balance_sheet: &BalanceSheets,
    profit_and_loss: &ProfitAndLossStatements,
    accounting_periods: &AccountingPeriods,
    chart_id: ChartId,
    chart_of_accounts_seed_path: PathBuf,
    accounting_init_config: AccountingInitConfig,
) -> Result<(), AccountingInitError> {
    let AccountingInitConfig {
        credit_config_path,
        deposit_config_path,
        balance_sheet_config_path,
        profit_and_loss_config_path,
        accounting_period_config_path,
        chart_of_accounts_opening_date: _,
        chart_of_accounts_seed_path: _,
    } = accounting_init_config;

    let data = std::fs::read_to_string(chart_of_accounts_seed_path)?;
    let chart = if let (chart, Some(new_account_set_ids)) = chart_of_accounts
        .import_from_csv(&Subject::System, chart_id, data)
        .await?
    {
        trial_balances
            .add_new_chart_accounts_to_trial_balance(
                TRIAL_BALANCE_STATEMENT_NAME,
                &new_account_set_ids,
            )
            .await?;
        chart
    } else {
        return Ok(());
    };

    if let Some(config_path) = credit_config_path {
        credit_module_configure(credit, &chart, config_path)
            .await
            .unwrap_or_else(|e| {
                dbg!(&e); // TODO: handle the un-returned error differently
            });
    }

    if let Some(config_path) = deposit_config_path {
        deposit_module_configure(deposit, &chart, config_path)
            .await
            .unwrap_or_else(|e| {
                dbg!(&e); // TODO: handle the un-returned error differently
            });
    }

    if let Some(config_path) = balance_sheet_config_path {
        balance_sheet_module_configure(balance_sheet, &chart, config_path)
            .await
            .unwrap_or_else(|e| {
                dbg!(&e); // TODO: handle the un-returned error differently
            });
    }

    if let Some(config_path) = profit_and_loss_config_path {
        profit_and_loss_module_configure(profit_and_loss, &chart, config_path)
            .await
            .unwrap_or_else(|e| {
                dbg!(&e); // TODO: handle the un-returned error differently
            });
    }

    if let Some(config_path) = accounting_period_config_path {
        accounting_period_module_configure(accounting_periods, &chart, config_path)
            .await
            .unwrap_or_else(|e| {
                dbg!(&e); // TODO: handle the un-returned error differently
            });
    }

    Ok(())
}
