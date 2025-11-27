use async_graphql::*;
use domain_configurations::DomainConfigurationRecord;

use crate::primitives::*;

pub use lana_app::profit_and_loss::ChartOfAccountsIntegrationConfig as DomainChartOfAccountsIntegrationConfig;

#[derive(SimpleObject, Clone)]
pub struct ProfitAndLossStatementModuleConfig {
    chart_of_accounts_id: Option<UUID>,
    chart_of_accounts_revenue_code: Option<String>,
    chart_of_accounts_cost_of_revenue_code: Option<String>,
    chart_of_accounts_expenses_code: Option<String>,

    #[graphql(skip)]
    pub(super) _entity: Arc<DomainChartOfAccountsIntegrationConfig>,
}

impl From<DomainChartOfAccountsIntegrationConfig> for ProfitAndLossStatementModuleConfig {
    fn from(value: DomainChartOfAccountsIntegrationConfig) -> Self {
        Self {
            chart_of_accounts_id: Some(value.chart_of_accounts_id.into()),
            chart_of_accounts_expenses_code: Some(
                value.chart_of_accounts_expenses_code.to_string(),
            ),
            chart_of_accounts_revenue_code: Some(value.chart_of_accounts_revenue_code.to_string()),
            chart_of_accounts_cost_of_revenue_code: Some(
                value.chart_of_accounts_cost_of_revenue_code.to_string(),
            ),
            _entity: Arc::new(value),
        }
    }
}

impl From<DomainConfigurationRecord<DomainChartOfAccountsIntegrationConfig>>
    for ProfitAndLossStatementModuleConfig
{
    fn from(record: DomainConfigurationRecord<DomainChartOfAccountsIntegrationConfig>) -> Self {
        Self::from(record.value)
    }
}

#[derive(InputObject)]
pub struct ProfitAndLossModuleConfigureInput {
    pub chart_of_accounts_revenue_code: String,
    pub chart_of_accounts_cost_of_revenue_code: String,
    pub chart_of_accounts_expenses_code: String,
}

crate::mutation_payload! { ProfitAndLossStatementModuleConfigurePayload, profit_and_loss_config: ProfitAndLossStatementModuleConfig }
