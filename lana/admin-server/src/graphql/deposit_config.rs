use async_graphql::*;

use crate::primitives::*;

pub use lana_app::deposit::ChartOfAccountsIntegrationConfig as DomainChartOfAccountsIntegrationConfig;

#[derive(SimpleObject, Clone)]
pub struct DepositModuleConfig {
    chart_of_accounts_id: Option<UUID>,
    chart_of_accounts_deposit_accounts_parent_code: Option<String>,
    chart_of_accounts_omnibus_parent_code: Option<String>,

    #[graphql(skip)]
    pub(super) _entity: Arc<DomainChartOfAccountsIntegrationConfig>,
}

impl From<DomainChartOfAccountsIntegrationConfig> for DepositModuleConfig {
    fn from(values: DomainChartOfAccountsIntegrationConfig) -> Self {
        Self {
            chart_of_accounts_id: Some(values.chart_of_accounts_id.into()),
            chart_of_accounts_deposit_accounts_parent_code: Some(
                values
                    .chart_of_accounts_deposit_accounts_parent_code
                    .to_string(),
            ),
            chart_of_accounts_omnibus_parent_code: Some(
                values.chart_of_accounts_omnibus_parent_code.to_string(),
            ),

            _entity: Arc::new(values),
        }
    }
}

#[derive(InputObject)]
pub struct DepositModuleConfigUpdateInput {
    pub chart_of_accounts_id: UUID,
    pub chart_of_accounts_deposit_accounts_parent_code: String,
    pub chart_of_accounts_omnibus_parent_code: String,
}
crate::mutation_payload! { DepositModuleConfigUpdatePayload, deposit_config: DepositModuleConfig }
