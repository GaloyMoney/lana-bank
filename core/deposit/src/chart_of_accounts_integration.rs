use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use chart_of_accounts::{AccountCode, ChartId};

#[derive(Builder, Debug, Serialize, Deserialize, Clone)]
pub struct ChartOfAccountsIntegrationConfig {
    #[builder(setter(into))]
    pub chart_of_accounts_id: ChartId,
    pub chart_of_accounts_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_omnibus_parent_code: AccountCode,
}

impl ChartOfAccountsIntegrationConfig {
    pub fn builder() -> ChartOfAccountsIntegrationConfigBuilder {
        ChartOfAccountsIntegrationConfigBuilder::default()
    }
}
