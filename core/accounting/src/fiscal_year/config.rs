use serde::{Deserialize, Serialize};

use crate::{
    ClosingAccountCodes,
    primitives::{AccountCode, AccountingBaseConfig},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FiscalYearConfig {
    pub revenue_account_code: String,
    pub cost_of_revenue_account_code: String,
    pub expenses_account_code: String,
    pub equity_retained_earnings_account_code: String,
    pub equity_retained_losses_account_code: String,
}

impl From<&AccountingBaseConfig> for FiscalYearConfig {
    fn from(config: &AccountingBaseConfig) -> Self {
        Self {
            revenue_account_code: config.revenue_code.to_string(),
            cost_of_revenue_account_code: config.cost_of_revenue_code.to_string(),
            expenses_account_code: config.expenses_code.to_string(),
            equity_retained_earnings_account_code: config
                .equity_retained_earnings_gain_code
                .to_string(),
            equity_retained_losses_account_code: config
                .equity_retained_earnings_loss_code
                .to_string(),
        }
    }
}

impl From<&FiscalYearConfig> for ClosingAccountCodes {
    fn from(config: &FiscalYearConfig) -> Self {
        Self {
            revenue: config
                .revenue_account_code
                .parse::<AccountCode>()
                .expect("Config was not validated"),
            cost_of_revenue: config
                .cost_of_revenue_account_code
                .parse::<AccountCode>()
                .expect("Config was not validated"),
            expenses: config
                .expenses_account_code
                .parse::<AccountCode>()
                .expect("Config was not validated"),
            equity_retained_earnings: config
                .equity_retained_earnings_account_code
                .parse::<AccountCode>()
                .expect("Config was not validated"),
            equity_retained_losses: config
                .equity_retained_losses_account_code
                .parse::<AccountCode>()
                .expect("Config was not validated"),
        }
    }
}
