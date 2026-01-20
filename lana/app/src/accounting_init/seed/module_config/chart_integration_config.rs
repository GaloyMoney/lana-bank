use serde::Deserialize;
use std::path::PathBuf;

use core_accounting::AccountingBaseConfig;

use crate::accounting_init::error::AccountingInitError;

#[derive(Debug, Clone, Deserialize)]
struct AccountingBaseConfigData {
    assets_code: String,
    liabilities_code: String,
    equity_code: String,
    equity_retained_earnings_gain_code: String,
    equity_retained_earnings_loss_code: String,
    revenue_code: String,
    cost_of_revenue_code: String,
    expenses_code: String,
}

impl TryFrom<AccountingBaseConfigData> for AccountingBaseConfig {
    type Error = AccountingInitError;

    fn try_from(data: AccountingBaseConfigData) -> Result<Self, Self::Error> {
        Ok(AccountingBaseConfig::try_new(
            data.assets_code.parse()?,
            data.liabilities_code.parse()?,
            data.equity_code.parse()?,
            data.equity_retained_earnings_gain_code.parse()?,
            data.equity_retained_earnings_loss_code.parse()?,
            data.revenue_code.parse()?,
            data.cost_of_revenue_code.parse()?,
            data.expenses_code.parse()?,
        )?)
    }
}

pub(in crate::accounting_init::seed) fn load_chart_integration_config_from_path(
    path: PathBuf,
) -> Result<AccountingBaseConfig, AccountingInitError> {
    let config_raw = std::fs::read_to_string(path)?;
    let config_data: AccountingBaseConfigData = serde_json::from_str(&config_raw)?;
    config_data.try_into()
}
