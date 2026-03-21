use serde::{Deserialize, Serialize};

use chart_primitives::{
    AccountCategory, AccountCode, CalaAccountSetId, ChartId, ChartLookup, ChartLookupError,
};
use domain_config::define_internal_config;

use super::error::ChartOfAccountsIntegrationError;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct ChartOfAccountsIntegrationConfig {
    pub chart_of_accounts_id: ChartId,
    pub trading_account_parent_code: AccountCode,
    pub rounding_account_parent_code: AccountCode,
    pub realized_gain_parent_code: AccountCode,
    pub realized_loss_parent_code: AccountCode,
}

define_internal_config! {
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub(crate) struct ResolvedChartOfAccountsIntegrationConfig {
        pub(crate) config: ChartOfAccountsIntegrationConfig,

        pub(crate) trading_account_parent_account_set_id: CalaAccountSetId,
        pub(crate) rounding_account_parent_account_set_id: CalaAccountSetId,
        pub(crate) realized_gain_parent_account_set_id: CalaAccountSetId,
        pub(crate) realized_loss_parent_account_set_id: CalaAccountSetId,
    }

    spec {
        key: "fx-chart-of-accounts-integration";
    }
}

impl ResolvedChartOfAccountsIntegrationConfig {
    pub(super) fn try_new(
        config: ChartOfAccountsIntegrationConfig,
        chart: &dyn ChartLookup,
    ) -> Result<Self, ChartOfAccountsIntegrationError> {
        let resolve = |code: &AccountCode,
                       category: AccountCategory|
         -> Result<CalaAccountSetId, ChartOfAccountsIntegrationError> {
            chart
                .find_account_set_id_in_category(code, category)
                .ok_or_else(|| {
                    ChartLookupError::InvalidAccountCategory {
                        code: code.clone(),
                        category,
                    }
                    .into()
                })
        };

        let trading_account_parent_account_set_id =
            resolve(&config.trading_account_parent_code, AccountCategory::Equity)?;
        let rounding_account_parent_account_set_id = resolve(
            &config.rounding_account_parent_code,
            AccountCategory::Equity,
        )?;
        let realized_gain_parent_account_set_id =
            resolve(&config.realized_gain_parent_code, AccountCategory::Revenue)?;
        let realized_loss_parent_account_set_id = resolve(
            &config.realized_loss_parent_code,
            AccountCategory::CostOfRevenue,
        )?;

        Ok(Self {
            config,
            trading_account_parent_account_set_id,
            rounding_account_parent_account_set_id,
            realized_gain_parent_account_set_id,
            realized_loss_parent_account_set_id,
        })
    }
}
