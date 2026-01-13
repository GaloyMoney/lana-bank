use async_graphql::*;

use crate::primitives::*;

pub use lana_app::primitives::AccountingBaseConfig as DomainAccountingBaseConfig;

#[derive(SimpleObject, Clone)]
pub struct BalanceSheetModuleConfig {
    chart_of_accounts_assets_code: Option<String>,
    chart_of_accounts_liabilities_code: Option<String>,
    chart_of_accounts_equity_code: Option<String>,
    chart_of_accounts_revenue_code: Option<String>,
    chart_of_accounts_cost_of_revenue_code: Option<String>,
    chart_of_accounts_expenses_code: Option<String>,

    #[graphql(skip)]
    pub(super) _entity: Arc<DomainAccountingBaseConfig>,
}

impl From<DomainAccountingBaseConfig> for BalanceSheetModuleConfig {
    fn from(values: DomainAccountingBaseConfig) -> Self {
        Self {
            chart_of_accounts_assets_code: Some(values.assets_code.to_string()),
            chart_of_accounts_liabilities_code: Some(values.liabilities_code.to_string()),
            chart_of_accounts_equity_code: Some(values.equity_code.to_string()),
            chart_of_accounts_revenue_code: Some(values.revenue_code.to_string()),
            chart_of_accounts_cost_of_revenue_code: Some(values.cost_of_revenue_code.to_string()),
            chart_of_accounts_expenses_code: Some(values.expenses_code.to_string()),

            _entity: Arc::new(values),
        }
    }
}

crate::mutation_payload! { BalanceSheetModuleConfigurePayload, balance_sheet_config: BalanceSheetModuleConfig }
