use async_graphql::*;
use chrono::{DateTime, Utc};
use domain_configurations::DomainConfigurationRecord;

use crate::primitives::*;

pub use lana_app::deposit::ChartOfAccountsIntegrationConfig as DomainChartOfAccountsIntegrationConfig;

#[derive(SimpleObject, Clone)]
pub struct DepositModuleConfig {
    chart_of_accounts_id: Option<UUID>,
    chart_of_accounts_omnibus_parent_code: Option<String>,
    chart_of_accounts_individual_deposit_accounts_parent_code: Option<String>,
    chart_of_accounts_government_entity_deposit_accounts_parent_code: Option<String>,
    chart_of_account_private_company_deposit_accounts_parent_code: Option<String>,
    chart_of_account_bank_deposit_accounts_parent_code: Option<String>,
    chart_of_account_financial_institution_deposit_accounts_parent_code: Option<String>,
    chart_of_account_non_domiciled_individual_deposit_accounts_parent_code: Option<String>,
    chart_of_accounts_frozen_individual_deposit_accounts_parent_code: Option<String>,
    chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code: Option<String>,
    chart_of_account_frozen_private_company_deposit_accounts_parent_code: Option<String>,
    chart_of_account_frozen_bank_deposit_accounts_parent_code: Option<String>,
    chart_of_account_frozen_financial_institution_deposit_accounts_parent_code: Option<String>,
    chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code: Option<String>,
    updated_by: Option<String>,
    updated_at: Option<DateTime<Utc>>,
    reason: Option<String>,
    correlation_id: Option<String>,

    #[graphql(skip)]
    pub(super) _entity: Arc<DomainChartOfAccountsIntegrationConfig>,
}

impl From<DomainConfigurationRecord<DomainChartOfAccountsIntegrationConfig>> for DepositModuleConfig {
    fn from(record: DomainConfigurationRecord<DomainChartOfAccountsIntegrationConfig>) -> Self {
        let values = record.value;
        Self {
            chart_of_accounts_id: Some(values.chart_of_accounts_id.into()),
            chart_of_accounts_omnibus_parent_code: Some(
                values.chart_of_accounts_omnibus_parent_code.to_string(),
            ),
            chart_of_accounts_individual_deposit_accounts_parent_code: Some(
                values
                    .chart_of_accounts_individual_deposit_accounts_parent_code
                    .to_string(),
            ),
            chart_of_accounts_government_entity_deposit_accounts_parent_code: Some(
                values
                    .chart_of_accounts_government_entity_deposit_accounts_parent_code
                    .to_string(),
            ),
            chart_of_account_private_company_deposit_accounts_parent_code: Some(
                values
                    .chart_of_account_private_company_deposit_accounts_parent_code
                    .to_string(),
            ),
            chart_of_account_bank_deposit_accounts_parent_code: Some(
                values
                    .chart_of_account_bank_deposit_accounts_parent_code
                    .to_string(),
            ),
            chart_of_account_financial_institution_deposit_accounts_parent_code: Some(
                values
                    .chart_of_account_financial_institution_deposit_accounts_parent_code
                    .to_string(),
            ),
            chart_of_account_non_domiciled_individual_deposit_accounts_parent_code: Some(
                values
                    .chart_of_account_non_domiciled_individual_deposit_accounts_parent_code
                    .to_string(),
            ),
            chart_of_accounts_frozen_individual_deposit_accounts_parent_code: Some(
                values
                    .chart_of_accounts_frozen_individual_deposit_accounts_parent_code
                    .to_string(),
            ),
            chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code: Some(
                values
                    .chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code
                    .to_string(),
            ),
            chart_of_account_frozen_private_company_deposit_accounts_parent_code: Some(
                values
                    .chart_of_account_frozen_private_company_deposit_accounts_parent_code
                    .to_string(),
            ),
            chart_of_account_frozen_bank_deposit_accounts_parent_code: Some(
                values
                    .chart_of_account_frozen_bank_deposit_accounts_parent_code
                    .to_string(),
            ),
            chart_of_account_frozen_financial_institution_deposit_accounts_parent_code: Some(
                values
                    .chart_of_account_frozen_financial_institution_deposit_accounts_parent_code
                    .to_string(),
            ),
            chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code: Some(
                values
                    .chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code
                    .to_string(),
            ),
            updated_by: Some(record.updated_by),
            updated_at: Some(record.updated_at),
            reason: record.reason,
            correlation_id: record.correlation_id,

            _entity: Arc::new(values),
        }
    }
}

#[derive(InputObject)]
pub struct DepositModuleConfigureInput {
    pub chart_of_accounts_omnibus_parent_code: String,
    pub chart_of_accounts_individual_deposit_accounts_parent_code: String,
    pub chart_of_accounts_government_entity_deposit_accounts_parent_code: String,
    pub chart_of_account_private_company_deposit_accounts_parent_code: String,
    pub chart_of_account_bank_deposit_accounts_parent_code: String,
    pub chart_of_account_financial_institution_deposit_accounts_parent_code: String,
    pub chart_of_account_non_domiciled_individual_deposit_accounts_parent_code: String,
    pub chart_of_accounts_frozen_individual_deposit_accounts_parent_code: String,
    pub chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code: String,
    pub chart_of_account_frozen_private_company_deposit_accounts_parent_code: String,
    pub chart_of_account_frozen_bank_deposit_accounts_parent_code: String,
    pub chart_of_account_frozen_financial_institution_deposit_accounts_parent_code: String,
    pub chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code: String,
}
crate::mutation_payload! { DepositModuleConfigurePayload, deposit_config: DepositModuleConfig }
