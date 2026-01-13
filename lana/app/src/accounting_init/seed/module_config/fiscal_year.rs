use std::{fs, path::PathBuf};

use serde::Deserialize;

use crate::{
    accounting::Chart,
    accounting_init::AccountingInitError,
    fiscal_year::{FiscalYears, error::FiscalYearError},
};

// TODO: Remove un-needed config.
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
    chart: &Chart,
    config_path: PathBuf,
) -> Result<(), AccountingInitError> {
    // TODO: Remove un-needed config.
    let data = fs::read_to_string(config_path)?;
    let FiscalYearConfigData {
        revenue_account_code: _,
        cost_of_revenue_account_code: _,
        expenses_account_code: _,
        equity_retained_earnings_gain_account_code: _,
        equity_retained_earnings_loss_account_code: _,
    } = serde_json::from_str(&data)?;

    match fiscal_year.configure(chart.id).await {
        Ok(_) => (),
        Err(FiscalYearError::FiscalYearConfigAlreadyExists) => (),
        Err(e) => return Err(e.into()),
    };

    Ok(())
}
