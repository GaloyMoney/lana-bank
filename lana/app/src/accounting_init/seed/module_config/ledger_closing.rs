use serde::Deserialize;

use crate::{
    accounting::Chart,
    accounting_init::AccountingInitError,
    ledger_closing::{ChartOfAccountsIntegrationConfig, LedgerClosings, error::LedgerClosingError},
};

#[derive(Deserialize)]
struct LedgerClosingConfigData {
    revenue_code: String,
    cost_of_revenue_code: String,
    expenses_code: String,
    equity_retained_earnings_code: String,
    equity_retained_losses_code: String,
}

pub(in crate::accounting_init::seed) async fn ledger_closing_module_configure(
    ledger_closing: &LedgerClosings,
    chart: &Chart,
    config_path: std::path::PathBuf,
) -> Result<(), AccountingInitError> {
    let data = std::fs::read_to_string(config_path)?;
    let LedgerClosingConfigData {
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

    match ledger_closing
        .set_chart_of_accounts_integration_config(
            &rbac_types::Subject::System,
            chart,
            config_values,
        )
        .await
    {
        Ok(_) => (),
        Err(LedgerClosingError::LedgerClosingIntegrationConfigAlreadyExists) => (),
        Err(e) => return Err(e.into()),
    };

    Ok(())
}
