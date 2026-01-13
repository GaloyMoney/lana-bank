use crate::primitives::*;
use async_graphql::*;
pub use lana_app::fiscal_year::FiscalYearConfig as DomainFiscalYearConfig;

#[derive(SimpleObject, Clone)]
pub struct FiscalYearModuleConfig {
    pub revenue_account_code: String,
    pub cost_of_revenue_account_code: String,
    pub expenses_account_code: String,
    pub equity_retained_earnings_account_code: String,
    pub equity_retained_losses_account_code: String,
}

impl From<DomainFiscalYearConfig> for FiscalYearModuleConfig {
    fn from(value: DomainFiscalYearConfig) -> Self {
        Self {
            revenue_account_code: value.revenue_account_code,
            cost_of_revenue_account_code: value.cost_of_revenue_account_code,
            expenses_account_code: value.expenses_account_code,
            equity_retained_earnings_account_code: value.equity_retained_earnings_account_code,
            equity_retained_losses_account_code: value.equity_retained_losses_account_code,
        }
    }
}

#[derive(InputObject)]
pub struct FiscalYearModuleConfigureInput {
    pub chart_id: UUID,
}

crate::mutation_payload! { FiscalYearModuleConfigurePayload, fiscal_year_config: FiscalYearModuleConfig }
