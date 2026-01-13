use std::{fs, path::PathBuf};

use serde::Deserialize;

use crate::{
    accounting::Chart,
    accounting_init::{AccountingInitError, constants::BALANCE_SHEET_NAME},
    balance_sheet::{BalanceSheets, ChartOfAccountsIntegrationConfig, error::BalanceSheetError},
};

use rbac_types::Subject;

#[derive(Deserialize)]
struct BalanceSheetConfigData {
    assets_code: String,
    liabilities_code: String,
    equity_code: String,
    revenue_code: String,
    cost_of_revenue_code: String,
    expenses_code: String,
}

pub(in crate::accounting_init::seed) async fn balance_sheet_module_configure(
    balance_sheet: &BalanceSheets,
    chart: &Chart,
    config_path: PathBuf,
) -> Result<(), AccountingInitError> {
    let data = fs::read_to_string(config_path)?;
    let BalanceSheetConfigData {
        assets_code,
        liabilities_code,
        equity_code,
        revenue_code,
        cost_of_revenue_code,
        expenses_code,
    } = serde_json::from_str(&data)?;

    let _config_values = ChartOfAccountsIntegrationConfig {
        chart_of_accounts_id: chart.id,
        chart_of_accounts_assets_code: assets_code.parse()?,
        chart_of_accounts_liabilities_code: liabilities_code.parse()?,
        chart_of_accounts_equity_code: equity_code.parse()?,
        chart_of_accounts_revenue_code: revenue_code.parse()?,
        chart_of_accounts_cost_of_revenue_code: cost_of_revenue_code.parse()?,
        chart_of_accounts_expenses_code: expenses_code.parse()?,
    };

    match balance_sheet
        .set_chart_of_accounts_integration_config(
            &Subject::System,
            BALANCE_SHEET_NAME.to_string(),
            chart,
        )
        .await
    {
        Ok(_) => (),
        Err(BalanceSheetError::BalanceSheetConfigAlreadyExists) => (),
        Err(e) => return Err(e.into()),
    };

    Ok(())
}
