use async_graphql::*;

use crate::primitives::*;

pub use lana_app::deposit::DepositConfig as DomainDepositConfig;

#[derive(SimpleObject, Clone)]
pub struct DepositConfig {
    id: ID,
    chart_of_accounts_id: UUID,
    chart_of_accounts_deposit_accounts_parent_code: String,
    chart_of_accounts_omnibus_parent_code: String,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainDepositConfig>,
}

impl From<DomainDepositConfig> for DepositConfig {
    fn from(deposit_config: DomainDepositConfig) -> Self {
        Self {
            id: deposit_config.id.to_global_id(),
            chart_of_accounts_id: deposit_config.values.chart_of_accounts_id.into(),
            chart_of_accounts_deposit_accounts_parent_code: deposit_config
                .values
                .chart_of_accounts_deposit_accounts_parent_code
                .to_string(),
            chart_of_accounts_omnibus_parent_code: deposit_config
                .values
                .chart_of_accounts_omnibus_parent_code
                .to_string(),

            entity: Arc::new(deposit_config),
        }
    }
}

#[derive(InputObject)]
pub struct DepositConfigUpdateInput {
    pub chart_of_accounts_id: UUID,
    pub chart_of_accounts_deposit_accounts_parent_code: String,
    pub chart_of_accounts_omnibus_parent_code: String,
}
crate::mutation_payload! { DepositConfigUpdatePayload, deposit_config: DepositConfig }
