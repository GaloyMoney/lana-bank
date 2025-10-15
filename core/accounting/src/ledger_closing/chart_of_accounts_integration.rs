use serde::{Deserialize, Serialize};

use core_accounting::{AccountCode, ChartId};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChartOfAccountsIntegrationConfig {
    pub chart_of_accounts_id: ChartId,
    pub chart_of_accounts_revenue_accounts_parent_code: AccountCode,
    pub chart_of_accounts_expense_accounts_parent_code: AccountCode,
    pub chart_of_accounts_cost_of_revenue_accounts_parent_code: AccountCode,
    pub chart_of_account_retained_earnings_accounts_parent_code: AccountCode,
    pub chart_of_account_retained_losses_accounts_parent_code: AccountCode,
    pub chart_of_accounts_root_account_set_closing_config_metadata_key: String,
}
