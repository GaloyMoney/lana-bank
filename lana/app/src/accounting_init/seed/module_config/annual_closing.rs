use std::{fs, path::PathBuf};

use serde::Deserialize;

use crate::{
    accounting::Chart,
    accounting_init::{AccountingInitError, constants::ANNUAL_CLOSING_NAME},
    annual_closing::{
        AnnualClosingTransactions, ChartOfAccountsIntegrationConfig,
        error::AnnualClosingTransactionError,
    },
};

use rbac_types::Subject;

#[derive(Deserialize)]
struct AnnualClosingConfigData {
    revenue_code: String,
    cost_of_revenue_code: String,
    expenses_code: String,
    equity_retained_earnings_code: String,
    equity_retained_losses_code: String,
}

pub(in crate::accounting_init::seed) async fn annual_closing_module_configure(
    annual_closing: &AnnualClosingTransactions,
    chart: &Chart,
    config_path: PathBuf,
) -> Result<(), AccountingInitError> {
    let data = fs::read_to_string(config_path)?;
    let AnnualClosingConfigData {
        revenue_code,
        cost_of_revenue_code,
        expenses_code,
        equity_retained_earnings_code,
        equity_retained_losses_code,
    } = serde_json::from_str(&data)?;

    let config_values = ChartOfAccountsIntegrationConfig {
        chart_of_accounts_id: chart.id,
        chart_of_accounts_revenue_code: revenue_code.parse()?,
        chart_of_accounts_cost_of_revenue_code: cost_of_revenue_code.parse()?,
        chart_of_accounts_expenses_code: expenses_code.parse()?,
        chart_of_accounts_equity_retained_earnings_code: equity_retained_earnings_code.parse()?,
        chart_of_accounts_equity_retained_losses_code: equity_retained_losses_code.parse()?,
    };

    match annual_closing
        .set_chart_of_accounts_integration_config(
            &Subject::System,
            ANNUAL_CLOSING_NAME.to_string(),
            chart,
            config_values,
        )
        .await
    {
        Ok(_) => (),
        Err(AnnualClosingTransactionError::AnnualClosingIntegrationConfigAlreadyExists) => (),
        Err(e) => return Err(e.into()),
    };

    Ok(())
}
