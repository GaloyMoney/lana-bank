use serde::{Deserialize, Serialize};

use crate::primitives::{AccountCode, ChartId};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChartOfAccountsIntegrationConfig {
    pub chart_of_accounts_id: ChartId,
    pub chart_of_accounts_revenue_code: AccountCode,
    pub chart_of_accounts_expenses_code: AccountCode,
    pub chart_of_accounts_cost_of_revenue_code: AccountCode,
    pub chart_of_accounts_equity_retained_earnings_code: AccountCode,
    pub chart_of_accounts_equity_retained_losses_code: AccountCode,
    // TODO: primitives.
    pub fiscal_year_end_month: u8,
    pub fiscal_month_end: String, // "start", "mid", "end"
    pub grace_period_days: u8, // 5
    pub extended_grace_period_days: u8, // 10
    pub extended_grace_period_after_months: [u8; 4], // [6, 12]
}
