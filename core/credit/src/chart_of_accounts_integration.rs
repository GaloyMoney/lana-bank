use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use core_accounting::{AccountCode, ChartId};

#[derive(Builder, Debug, Serialize, Deserialize, Clone)]
pub struct ChartOfAccountsIntegrationConfig {
    #[builder(setter(into))]
    pub chart_of_accounts_id: ChartId,
    pub chart_of_account_facility_omnibus_parent_code: AccountCode,
    pub chart_of_account_collateral_omnibus_parent_code: AccountCode,
    pub chart_of_account_facility_parent_code: AccountCode,
    pub chart_of_account_collateral_parent_code: AccountCode,
    pub chart_of_account_interest_income_parent_code: AccountCode,
    pub chart_of_account_fee_income_parent_code: AccountCode,

    pub chart_of_account_short_term_individual_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_short_term_government_entity_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_short_term_private_company_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_short_term_bank_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code:
        AccountCode,
    pub chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
        AccountCode,
    pub chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code:
        AccountCode,

    pub chart_of_account_long_term_individual_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_long_term_government_entity_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_long_term_private_company_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_long_term_bank_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code:
        AccountCode,
    pub chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
        AccountCode,
    pub chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code:
        AccountCode,

    pub chart_of_account_short_term_individual_interest_receivable_parent_code: AccountCode,
    pub chart_of_account_short_term_government_entity_interest_receivable_parent_code: AccountCode,
    pub chart_of_account_short_term_private_company_interest_receivable_parent_code: AccountCode,
    pub chart_of_account_short_term_bank_interest_receivable_parent_code: AccountCode,
    pub chart_of_account_short_term_financial_institution_interest_receivable_parent_code:
        AccountCode,
    pub chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
        AccountCode,
    pub chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code:
        AccountCode,

    pub chart_of_account_long_term_individual_interest_receivable_parent_code: AccountCode,
    pub chart_of_account_long_term_government_entity_interest_receivable_parent_code: AccountCode,
    pub chart_of_account_long_term_private_company_interest_receivable_parent_code: AccountCode,
    pub chart_of_account_long_term_bank_interest_receivable_parent_code: AccountCode,
    pub chart_of_account_long_term_financial_institution_interest_receivable_parent_code:
        AccountCode,
    pub chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
        AccountCode,
    pub chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code:
        AccountCode,

    pub chart_of_account_overdue_individual_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_overdue_government_entity_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_overdue_private_company_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_overdue_bank_disbursed_receivable_parent_code: AccountCode,
    pub chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code:
        AccountCode,
    pub chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
        AccountCode,
    pub chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code:
        AccountCode,
}

impl ChartOfAccountsIntegrationConfig {
    pub fn builder() -> ChartOfAccountsIntegrationConfigBuilder {
        ChartOfAccountsIntegrationConfigBuilder::default()
    }
}
