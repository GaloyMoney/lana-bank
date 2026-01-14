use serde::{Deserialize, Serialize};

use domain_config::define_internal_config;

use crate::primitives::{AccountCode, AccountingBaseConfig};

define_internal_config! {
    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub struct BalanceSheetConfig {
        pub assets_code: AccountCode,
        pub liabilities_code: AccountCode,
        pub equity_code: AccountCode,
        pub equity_retained_earnings_gain_code: AccountCode,
        pub equity_retained_earnings_loss_code: AccountCode,
        pub revenue_code: AccountCode,
        pub cost_of_revenue_code: AccountCode,
        pub expenses_code: AccountCode,
    }

    spec {
        key: "balance-sheet";
    }
}

impl From<&AccountingBaseConfig> for BalanceSheetConfig {
    fn from(config: &AccountingBaseConfig) -> Self {
        Self {
            assets_code: config.assets_code.clone(),
            liabilities_code: config.liabilities_code.clone(),
            equity_code: config.equity_code.clone(),
            equity_retained_earnings_gain_code: config.equity_retained_earnings_gain_code.clone(),
            equity_retained_earnings_loss_code: config.equity_retained_earnings_loss_code.clone(),
            revenue_code: config.revenue_code.clone(),
            cost_of_revenue_code: config.cost_of_revenue_code.clone(),
            expenses_code: config.expenses_code.clone(),
        }
    }
}

impl From<&BalanceSheetConfig> for AccountingBaseConfig {
    fn from(config: &BalanceSheetConfig) -> Self {
        Self {
            assets_code: config.assets_code.clone(),
            liabilities_code: config.liabilities_code.clone(),
            equity_code: config.equity_code.clone(),
            equity_retained_earnings_gain_code: config.equity_retained_earnings_gain_code.clone(),
            equity_retained_earnings_loss_code: config.equity_retained_earnings_loss_code.clone(),
            revenue_code: config.revenue_code.clone(),
            cost_of_revenue_code: config.cost_of_revenue_code.clone(),
            expenses_code: config.expenses_code.clone(),
        }
    }
}
