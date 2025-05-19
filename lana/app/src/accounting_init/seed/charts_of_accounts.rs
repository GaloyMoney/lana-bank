use std::path::PathBuf;

use crate::{
    accounting::ChartId,
    accounting_init::{constants::*, *},
};

use rbac_types::Subject;

use super::module_config::{balance_sheet::*, credit::*, deposit::*, profit_and_loss::*};

pub(crate) async fn init(
    chart_of_accounts: &ChartOfAccounts,
    trial_balances: &TrialBalances,
    credit: &Credit,
    deposit: &Deposits,
    balance_sheet: &BalanceSheets,
    profit_and_loss: &ProfitAndLossStatements,
    accounting_init_config: AccountingInitConfig,
) -> Result<(), AccountingInitError> {
    let chart_id = create_chart_of_accounts(chart_of_accounts).await?;

    if let Some(path) = accounting_init_config.clone().chart_of_accounts_seed_path {
        seed_chart_of_accounts(
            chart_of_accounts,
            trial_balances,
            credit,
            deposit,
            balance_sheet,
            profit_and_loss,
            chart_id,
            path,
            accounting_init_config,
        )
        .await?;
    }
    Ok(())
}

async fn create_chart_of_accounts(
    chart_of_accounts: &ChartOfAccounts,
) -> Result<ChartId, AccountingInitError> {
    if let Some(chart) = chart_of_accounts.find_by_reference(CHART_REF).await? {
        Ok(chart.id)
    } else {
        Ok(chart_of_accounts
            .create_chart(
                &Subject::System,
                CHART_NAME.to_string(),
                CHART_REF.to_string(),
            )
            .await?
            .id)
    }
}

#[allow(clippy::too_many_arguments)]
async fn seed_chart_of_accounts(
    chart_of_accounts: &ChartOfAccounts,
    trial_balances: &TrialBalances,
    credit: &Credit,
    deposit: &Deposits,
    balance_sheet: &BalanceSheets,
    profit_and_loss: &ProfitAndLossStatements,
    chart_id: ChartId,
    chart_of_accounts_seed_path: PathBuf,
    accounting_init_config: AccountingInitConfig,
) -> Result<(), AccountingInitError> {
    let AccountingInitConfig {
        credit_config_path,
        deposit_config_path,
        balance_sheet_config_path,
        profit_and_loss_config_path,

        chart_of_accounts_seed_path: _,
    } = accounting_init_config;

    let data = std::fs::read_to_string(chart_of_accounts_seed_path)?;
    if let Some(new_account_set_ids) = chart_of_accounts
        .import_from_csv(&Subject::System, chart_id, data)
        .await?
    {
        trial_balances
            .add_new_chart_accounts_to_trial_balance(
                TRIAL_BALANCE_STATEMENT_NAME,
                new_account_set_ids,
            )
            .await?;
    } else {
        return Ok(());
    }

    let chart = chart_of_accounts.find_by_id(chart_id).await?;

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

    Ok(())
}
