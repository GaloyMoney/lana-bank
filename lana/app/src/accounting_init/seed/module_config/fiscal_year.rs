use std::{fs, path::PathBuf};

use serde::Deserialize;

use crate::{
    accounting_init::AccountingInitError,
    fiscal_year::{FiscalYearConfig, FiscalYears, error::FiscalYearError},
};

#[derive(Deserialize)]
struct FiscalYearConfigData {
    revenue_account_code: String,
    cost_of_revenue_account_code: String,
    expenses_account_code: String,
    equity_retained_earnings_gain_account_code: String,
    equity_retained_earnings_loss_account_code: String,
}

pub(in crate::accounting_init::seed) async fn fiscal_year_module_configure(
    fiscal_year: &FiscalYears,
    config_path: PathBuf,
) -> Result<(), AccountingInitError> {
    let data = fs::read_to_string(config_path)?;
    let FiscalYearConfigData {
        revenue_account_code,
        cost_of_revenue_account_code,
        expenses_account_code,
        equity_retained_earnings_gain_account_code,
        equity_retained_earnings_loss_account_code,
    } = serde_json::from_str(&data)?;

    let config_values = FiscalYearConfig {
        revenue_account_code,
        cost_of_revenue_account_code,
        expenses_account_code,
        equity_retained_earnings_account_code: equity_retained_earnings_gain_account_code,
        equity_retained_losses_account_code: equity_retained_earnings_loss_account_code,
    };

    match fiscal_year.configure(config_values).await {
        Ok(_) => (),
        Err(FiscalYearError::FiscalYearConfigAlreadyExists) => (),
        Err(e) => return Err(e.into()),
    };

    Ok(())
}
