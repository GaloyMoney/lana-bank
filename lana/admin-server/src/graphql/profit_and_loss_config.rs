use async_graphql::*;

use crate::primitives::*;

pub use lana_app::primitives::AccountingBaseConfig as DomainAccountingBaseConfig;

#[derive(SimpleObject, Clone)]
pub struct ProfitAndLossStatementModuleConfig {
    chart_of_accounts_revenue_code: Option<String>,
    chart_of_accounts_cost_of_revenue_code: Option<String>,
    chart_of_accounts_expenses_code: Option<String>,

    #[graphql(skip)]
    pub(super) _entity: Arc<DomainAccountingBaseConfig>,
}

impl From<DomainAccountingBaseConfig> for ProfitAndLossStatementModuleConfig {
    fn from(value: DomainAccountingBaseConfig) -> Self {
        Self {
            chart_of_accounts_expenses_code: Some(value.expenses_code.to_string()),
            chart_of_accounts_revenue_code: Some(value.revenue_code.to_string()),
            chart_of_accounts_cost_of_revenue_code: Some(value.cost_of_revenue_code.to_string()),
            _entity: Arc::new(value),
        }
    }
}

#[derive(InputObject)]
pub struct ProfitAndLossModuleConfigureInput {
    pub chart_of_accounts_revenue_code: String,
    pub chart_of_accounts_cost_of_revenue_code: String,
    pub chart_of_accounts_expenses_code: String,
}

crate::mutation_payload! { ProfitAndLossStatementModuleConfigurePayload, profit_and_loss_config: ProfitAndLossStatementModuleConfig }
