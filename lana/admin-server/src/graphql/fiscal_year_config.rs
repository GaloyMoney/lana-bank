use async_graphql::*;

use crate::primitives::*;

pub use lana_app::fiscal_year::FiscalYearConfig as DomainFiscalYearConfig;

#[derive(SimpleObject, Clone)]
pub struct FiscalYearModuleConfig {
    year_end_month: Option<u32>,
    revenue_code: Option<String>,
    cost_of_revenue_code: Option<String>,
    expenses_code: Option<String>,
    equity_retained_earnings_code: Option<String>,
    equity_retained_losses_code: Option<String>,

    #[graphql(skip)]
    pub(super) _entity: Arc<DomainFiscalYearConfig>,
}

impl From<DomainFiscalYearConfig> for FiscalYearModuleConfig {
    fn from(value: DomainFiscalYearConfig) -> Self {
        Self {
            year_end_month: Some(value.year_end_month),
            revenue_code: Some(value.revenue_code.to_string()),
            cost_of_revenue_code: Some(value.cost_of_revenue_code.to_string()),
            expenses_code: Some(value.expenses_code.to_string()),
            equity_retained_earnings_code: Some(value.equity_retained_earnings_code.to_string()),
            equity_retained_losses_code: Some(value.equity_retained_losses_code.to_string()),
            _entity: Arc::new(value),
        }
    }
}

#[derive(InputObject)]
pub struct FiscalYearModuleConfigureInput {
    pub year_end_month: u32,
    pub revenue_code: String,
    pub cost_of_revenue_code: String,
    pub expenses_code: String,
    pub equity_retained_earnings_code: String,
    pub equity_retained_losses_code: String,
}

crate::mutation_payload! { FiscalYearModuleConfigurePayload, fiscal_year_config: FiscalYearModuleConfig }
