use serde::{Deserialize, Serialize};

use crate::primitives::{AccountCode, ChartId};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChartOfAccountsIntegrationConfig {
    pub chart_of_accounts_id: ChartId,
    pub chart_of_accounts_revenue_code: AccountCode,
    pub chart_of_accounts_cost_of_revenue_code: AccountCode,
    pub chart_of_accounts_expenses_code: AccountCode,
    pub chart_of_accounts_equity_retained_earnings_code: AccountCode,
    pub chart_of_accounts_equity_retained_losses_code: AccountCode,

    pub accounting_period_monthly: Monthly,
    pub accounting_period_annually: Annually,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Monthly {
    #[serde(flatten)]
    pub basis: MonthlyBasis,
    pub grace_period_days: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "basis", rename_all = "lowercase")]
pub enum MonthlyBasis {
    Calendar,
    OnDay { on_day: u8 },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Annually {
    #[serde(flatten)]
    pub basis: AnnuallyBasis,
    pub grace_period_days: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "basis", rename_all = "lowercase")]
pub enum AnnuallyBasis {
    Calendar,
    Fiscal { on_month: u8, on_day: u8 },
}
