use serde::Deserialize;
use std::path::PathBuf;

use core_accounting::AccountingBaseConfig;

use crate::accounting_init::error::AccountingInitError;

#[derive(Debug, Clone, Deserialize)]
pub struct AccountingBaseConfigData {
    pub assets_code: String,
    pub liabilities_code: String,
    pub equity_code: String,
    pub equity_retained_earnings_gain_code: String,
    pub equity_retained_earnings_loss_code: String,
    pub revenue_code: String,
    pub cost_of_revenue_code: String,
    pub expenses_code: String,
}

impl TryFrom<AccountingBaseConfigData> for AccountingBaseConfig {
    type Error = AccountingInitError;

    fn try_from(data: AccountingBaseConfigData) -> Result<Self, Self::Error> {
        Ok(AccountingBaseConfig {
            assets_code: data.assets_code.parse()?,
            liabilities_code: data.liabilities_code.parse()?,
            equity_code: data.equity_code.parse()?,
            equity_retained_earnings_gain_code: data.equity_retained_earnings_gain_code.parse()?,
            equity_retained_earnings_loss_code: data.equity_retained_earnings_loss_code.parse()?,
            revenue_code: data.revenue_code.parse()?,
            cost_of_revenue_code: data.cost_of_revenue_code.parse()?,
            expenses_code: data.expenses_code.parse()?,
        })
    }
}

pub(in crate::accounting_init::seed) fn load_chart_integration_config_from_path(
    path: PathBuf,
) -> Result<AccountingBaseConfig, AccountingInitError> {
    let config_raw = std::fs::read_to_string(path)?;
    let config_data: AccountingBaseConfigData = serde_json::from_str(&config_raw)?;
    config_data.try_into()
}
