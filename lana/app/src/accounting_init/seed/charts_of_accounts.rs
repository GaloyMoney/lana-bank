use crate::accounting_init::{constants::*, *};

use rbac_types::Subject;

use super::module_config::{
    balance_sheet::*, credit::*, deposit::*, fiscal_year::*, profit_and_loss::*,
};

pub(crate) async fn init(
    chart_of_accounts: &ChartOfAccounts,
    trial_balances: &TrialBalances,
    credit: &Credit,
    deposit: &Deposits,
    balance_sheet: &BalanceSheets,
    profit_and_loss: &ProfitAndLossStatements,
    fiscal_year: &FiscalYears,
    accounting_init_config: AccountingInitConfig,
) -> Result<(), AccountingInitError> {
    create_chart_of_accounts(
        chart_of_accounts,
        fiscal_year,
        accounting_init_config.clone(),
    )
    .await?;

    seed_chart_of_accounts(
        chart_of_accounts,
        trial_balances,
        credit,
        deposit,
        balance_sheet,
        profit_and_loss,
        fiscal_year,
        accounting_init_config,
    )
    .await?;

    Ok(())
}

async fn create_chart_of_accounts(
    chart_of_accounts: &ChartOfAccounts,
    fiscal_year: &FiscalYears,
    accounting_init_config: AccountingInitConfig,
) -> Result<(), AccountingInitError> {
    let opening_date = accounting_init_config
        .chart_of_accounts_opening_date
        .ok_or_else(|| {
            AccountingInitError::MissingConfig("chart_of_accounts_opening_date".to_string())
        })?;
    if chart_of_accounts
        .maybe_find_by_reference(CHART_REF)
        .await?
        .is_none()
    {
        let chart = chart_of_accounts
            .create_chart(
                &Subject::System,
                CHART_NAME.to_string(),
                CHART_REF.to_string(),
            )
            .await?;

        fiscal_year
            .init_for_chart(&Subject::System, opening_date, chart.id)
            .await?;
    }

    Ok(())
}

async fn seed_chart_of_accounts(
    chart_of_accounts: &ChartOfAccounts,
    trial_balances: &TrialBalances,
    credit: &Credit,
    deposit: &Deposits,
    balance_sheet: &BalanceSheets,
    profit_and_loss: &ProfitAndLossStatements,
    fiscal_year: &FiscalYears,
    accounting_init_config: AccountingInitConfig,
) -> Result<(), AccountingInitError> {
    let AccountingInitConfig {
        chart_of_accounts_seed_path: seed_path,

        credit_config_path,
        deposit_config_path,
        balance_sheet_config_path,
        profit_and_loss_config_path,
        fiscal_year_config_path,
        chart_of_accounts_opening_date: _,
    } = accounting_init_config;

    let data = match seed_path {
        Some(seed_path) => std::fs::read_to_string(seed_path)?,
        None => return Ok(()),
    };

    let chart = if let (chart, Some(new_account_set_ids)) = chart_of_accounts
        .import_from_csv(&Subject::System, CHART_REF, data)
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

    if let Some(config_path) = fiscal_year_config_path {
        fiscal_year_module_configure(fiscal_year, config_path)
            .await
            .unwrap_or_else(|e| {
                dbg!(&e); // TODO: handle the un-returned error differently
            });
    }

    Ok(())
}
