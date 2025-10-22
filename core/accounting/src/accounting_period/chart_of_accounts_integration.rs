use serde::{Deserialize, Serialize};

use crate::primitives::{AccountCode, ChartId};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChartOfAccountsIntegrationConfig {
    pub chart_of_accounts_id: ChartId,
    pub revenue_code: AccountCode,
    pub cost_of_revenue_code: AccountCode,
    pub expenses_code: AccountCode,
    pub equity_retained_earnings_code: AccountCode,
    pub equity_retained_losses_code: AccountCode,
    pub accounting_periods: Vec<AccountingPeriodConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountingPeriodConfig {
    #[serde(flatten)]
    pub basis: Basis,
    pub grace_period_days: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "basis", rename_all = "lowercase")]
pub enum Basis {
    Monthly { on_day: u8 },
    Annual { on_month: u8, on_day: u8 },
}
