use async_graphql::*;

use crate::primitives::*;

pub use lana_app::credit_facility::ChartOfAccountsIntegrationConfig as DomainChartOfAccountsIntegrationConfig;

#[derive(SimpleObject, Clone)]
pub struct CreditModuleConfig {
    chart_of_accounts_id: Option<UUID>,
    chart_of_account_facility_omnibus_parent_code: Option<String>,
    chart_of_account_collateral_omnibus_parent_code: Option<String>,
    chart_of_account_facility_parent_code: Option<String>,
    chart_of_account_collateral_parent_code: Option<String>,
    chart_of_account_disbursed_receivable_parent_code: Option<String>,
    chart_of_account_interest_receivable_parent_code: Option<String>,
    chart_of_account_interest_income_parent_code: Option<String>,
    chart_of_account_fee_income_parent_code: Option<String>,

    #[graphql(skip)]
    pub(super) _entity: Arc<DomainChartOfAccountsIntegrationConfig>,
}

impl From<DomainChartOfAccountsIntegrationConfig> for CreditModuleConfig {
    fn from(values: DomainChartOfAccountsIntegrationConfig) -> Self {
        Self {
            chart_of_accounts_id: Some(values.chart_of_accounts_id.into()),
            chart_of_account_facility_omnibus_parent_code: Some(
                values
                    .chart_of_account_facility_omnibus_parent_code
                    .to_string(),
            ),
            chart_of_account_collateral_omnibus_parent_code: Some(
                values
                    .chart_of_account_collateral_omnibus_parent_code
                    .to_string(),
            ),
            chart_of_account_facility_parent_code: Some(
                values.chart_of_account_facility_parent_code.to_string(),
            ),
            chart_of_account_collateral_parent_code: Some(
                values.chart_of_account_collateral_parent_code.to_string(),
            ),
            chart_of_account_disbursed_receivable_parent_code: Some(
                values
                    .chart_of_account_disbursed_receivable_parent_code
                    .to_string(),
            ),
            chart_of_account_interest_receivable_parent_code: Some(
                values
                    .chart_of_account_interest_receivable_parent_code
                    .to_string(),
            ),
            chart_of_account_interest_income_parent_code: Some(
                values
                    .chart_of_account_interest_income_parent_code
                    .to_string(),
            ),
            chart_of_account_fee_income_parent_code: Some(
                values.chart_of_account_fee_income_parent_code.to_string(),
            ),
            _entity: Arc::new(values),
        }
    }
}

#[derive(InputObject)]
pub struct CreditModuleConfigUpdateInput {
    pub chart_of_account_facility_omnibus_parent_code: String,
    pub chart_of_account_collateral_omnibus_parent_code: String,
    pub chart_of_account_facility_parent_code: String,
    pub chart_of_account_collateral_parent_code: String,
    pub chart_of_account_disbursed_receivable_parent_code: String,
    pub chart_of_account_interest_receivable_parent_code: String,
    pub chart_of_account_interest_income_parent_code: String,
    pub chart_of_account_fee_income_parent_code: String,
}
crate::mutation_payload! { CreditModuleConfigUpdatePayload, credit_config: CreditModuleConfig }
