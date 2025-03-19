use async_graphql::*;

use crate::primitives::*;

pub use lana_app::balance_sheet::ChartOfAccountsIntegrationConfig as DomainChartOfAccountsIntegrationConfig;

#[derive(SimpleObject, Clone)]
pub struct BalanceSheetModuleConfig {
    chart_of_accounts_id: Option<UUID>,
    chart_of_accounts_assets_code: Option<String>,
    chart_of_accounts_liabilities_code: Option<String>,
    chart_of_accounts_equity_code: Option<String>,
    chart_of_accounts_revenue_code: Option<String>,
    chart_of_accounts_cost_of_revenue_code: Option<String>,
    chart_of_accounts_expenses_code: Option<String>,

    #[graphql(skip)]
    pub(super) _entity: Arc<DomainChartOfAccountsIntegrationConfig>,
}

impl From<DomainChartOfAccountsIntegrationConfig> for BalanceSheetModuleConfig {
    fn from(values: DomainChartOfAccountsIntegrationConfig) -> Self {
        Self {
            chart_of_accounts_id: Some(values.chart_of_accounts_id.into()),
            chart_of_accounts_assets_code: Some(values.chart_of_accounts_assets_code.to_string()),
            chart_of_accounts_liabilities_code: Some(
                values.chart_of_accounts_liabilities_code.to_string(),
            ),
            chart_of_accounts_equity_code: Some(values.chart_of_accounts_equity_code.to_string()),
            chart_of_accounts_revenue_code: Some(values.chart_of_accounts_revenue_code.to_string()),
            chart_of_accounts_cost_of_revenue_code: Some(
                values.chart_of_accounts_cost_of_revenue_code.to_string(),
            ),
            chart_of_accounts_expenses_code: Some(
                values.chart_of_accounts_expenses_code.to_string(),
            ),

            _entity: Arc::new(values),
        }
    }
}

#[derive(InputObject)]
pub struct BalanceSheetModuleConfigureInput {
    pub chart_of_accounts_assets_code: String,
    pub chart_of_accounts_liabilities_code: String,
    pub chart_of_accounts_equity_code: String,
    pub chart_of_accounts_revenue_code: String,
    pub chart_of_accounts_cost_of_revenue_code: String,
    pub chart_of_accounts_expenses_code: String,
}
crate::mutation_payload! { BalanceSheetModuleConfigurePayload, balance_sheet_config: BalanceSheetModuleConfig }
