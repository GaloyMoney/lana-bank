use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use chart_of_accounts::{AccountCode, ChartId};

#[derive(Builder, Debug, Serialize, Deserialize, Clone)]
pub struct ChartOfAccountsIntegrationConfig {
    #[builder(setter(into))]
    pub chart_of_accounts_id: ChartId,
    pub chart_of_account_facility_omnibus_parent_code: AccountCode,
    pub chart_of_account_collateral_omnibus_parent_code: AccountCode,
    pub chart_of_account_facility_parent_code: AccountCode,
    pub chart_of_account_collateral_parent_code: AccountCode,
    pub chart_of_account_interest_receivable_parent_code: AccountCode,
    pub chart_of_account_interest_income_parent_code: AccountCode,
    pub chart_of_account_fee_income_parent_code: AccountCode,

    pub chart_of_account_individual_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_government_entity_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_private_company_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_bank_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_financial_institution_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_foreign_agency_or_subsidiary_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_non_domiciled_company_disbursed_receivable_parent_code: AccountCode,
}

impl ChartOfAccountsIntegrationConfig {
    pub fn builder() -> ChartOfAccountsIntegrationConfigBuilder {
        ChartOfAccountsIntegrationConfigBuilder::default()
    }
}
