use async_graphql::*;

use crate::primitives::*;

pub use lana_app::deposit::DepositConfig as DomainDepositConfig;

#[derive(SimpleObject, Clone)]
pub struct DepositConfig {
    id: ID,
    chart_of_accounts_id: Option<UUID>,
    chart_of_accounts_deposit_accounts_parent_code: Option<String>,
    chart_of_accounts_omnibus_parent_code: Option<String>,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainDepositConfig>,
}

impl From<DomainDepositConfig> for DepositConfig {
    fn from(deposit_config: DomainDepositConfig) -> Self {
        match deposit_config.values() {
            Ok(values) => Self {
                id: deposit_config.id.to_global_id(),
                chart_of_accounts_id: Some(values.chart_of_accounts_id.into()),
                chart_of_accounts_deposit_accounts_parent_code: Some(
                    values
                        .chart_of_accounts_deposit_accounts_parent_code
                        .to_string(),
                ),
                chart_of_accounts_omnibus_parent_code: Some(
                    values.chart_of_accounts_omnibus_parent_code.to_string(),
                ),

                entity: Arc::new(deposit_config),
            },
            Err(_) => Self {
                id: deposit_config.id.to_global_id(),
                chart_of_accounts_id: None,
                chart_of_accounts_deposit_accounts_parent_code: None,
                chart_of_accounts_omnibus_parent_code: None,

                entity: Arc::new(deposit_config),
            },
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
