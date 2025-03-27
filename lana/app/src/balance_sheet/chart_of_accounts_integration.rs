use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use core_accounting::{AccountCode, ChartId};

#[derive(Builder, Debug, Serialize, Deserialize, Clone)]
pub struct ChartOfAccountsIntegrationConfig {
    #[builder(setter(into))]
    pub chart_of_accounts_id: ChartId,
    pub chart_of_accounts_assets_code: AccountCode,
    pub chart_of_accounts_liabilities_code: AccountCode,
    pub chart_of_accounts_equity_code: AccountCode,
    pub chart_of_accounts_revenue_code: AccountCode,
    pub chart_of_accounts_cost_of_revenue_code: AccountCode,
    pub chart_of_accounts_expenses_code: AccountCode,
}

impl ChartOfAccountsIntegrationConfig {
    pub fn builder() -> ChartOfAccountsIntegrationConfigBuilder {
        ChartOfAccountsIntegrationConfigBuilder::default()
    }
}
