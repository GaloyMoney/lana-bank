use serde::{Deserialize, Serialize};

use domain_config::{DomainConfigError, define_internal_config};

use crate::{ClosingAccountCodes, primitives::AccountCode};

define_internal_config! {
    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub struct FiscalYearConfig {
        pub revenue_account_code: String,
        pub cost_of_revenue_account_code: String,
        pub expenses_account_code: String,
        pub equity_retained_earnings_account_code: String,
        pub equity_retained_losses_account_code: String,
    }

    spec {
        key: "fiscal-year";
        validate: |value: &Self| {
            if value.revenue_account_code.parse::<AccountCode>().is_err() {
                return Err(DomainConfigError::InvalidState(
                    "revenue_account_code should be a valid account code".to_string(),
                ));
            }

            if value
                .cost_of_revenue_account_code
                .parse::<AccountCode>()
                .is_err()
            {
                return Err(DomainConfigError::InvalidState(
                    "cost_of_revenue_account_code should be a valid account code".to_string(),
                ));
            }

            if value.expenses_account_code.parse::<AccountCode>().is_err() {
                return Err(DomainConfigError::InvalidState(
                    "expenses_account_code should be a valid account code".to_string(),
                ));
            }

            if value
                .equity_retained_earnings_account_code
                .parse::<AccountCode>()
                .is_err()
            {
                return Err(DomainConfigError::InvalidState(
                    "equity_retained_earnings_account_code should be a valid account code"
                        .to_string(),
                ));
            }

            if value
                .equity_retained_losses_account_code
                .parse::<AccountCode>()
                .is_err()
            {
                return Err(DomainConfigError::InvalidState(
                    "equity_retained_losses_account_code should be a valid account code"
                        .to_string(),
                ));
            }

            Ok(())
        };
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
