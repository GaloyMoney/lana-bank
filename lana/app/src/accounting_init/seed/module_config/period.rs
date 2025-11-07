use core_accounting::accounting_period::chart_of_accounts_integration::AccountingPeriodConfig;
use serde::Deserialize;

use crate::{
    accounting::Chart,
    accounting_init::AccountingInitError,
    accounting_period::{
        AccountingPeriods, ChartOfAccountsIntegrationConfig, error::AccountingPeriodError,
    },
};

#[derive(Deserialize)]
struct AccountingPeriodConfigData {
    revenue_code: String,
    cost_of_revenue_code: String,
    expenses_code: String,
    equity_retained_earnings_code: String,
    equity_retained_losses_code: String,
    accounting_periods: Vec<AccountingPeriodConfig>,
}

pub(in crate::accounting_init::seed) async fn accounting_period_module_configure(
    accounting_periods: &AccountingPeriods,
    chart: &Chart,
    config_path: std::path::PathBuf,
) -> Result<(), AccountingInitError> {
    let data = std::fs::read_to_string(config_path)?;
    let AccountingPeriodConfigData {
        revenue_code,
        cost_of_revenue_code,
        expenses_code,
        equity_retained_earnings_code,
        equity_retained_losses_code,
        accounting_periods: initial_periods,
    } = serde_json::from_str(&data)?;

    let config_values = ChartOfAccountsIntegrationConfig {
        chart_of_accounts_id: chart.id,
        revenue_code: revenue_code.parse()?,
        cost_of_revenue_code: cost_of_revenue_code.parse()?,
        expenses_code: expenses_code.parse()?,
        equity_retained_earnings_code: equity_retained_earnings_code.parse()?,
        equity_retained_losses_code: equity_retained_losses_code.parse()?,
        accounting_periods: initial_periods.clone(),
    };

    match accounting_periods
        .set_chart_of_accounts_integration_config(
            &rbac_types::Subject::System,
            chart,
            config_values,
        )
        .await
    {
        Ok(_) => (),
        Err(AccountingPeriodError::AccountingPeriodIntegrationConfigAlreadyExists) => (),
        Err(e) => return Err(e.into()),
    };

    Ok(())
}
