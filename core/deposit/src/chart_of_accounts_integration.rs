use serde::{Deserialize, Serialize};

use core_accounting::{AccountCode, ChartId};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChartOfAccountsIntegrationConfig {
    pub chart_of_accounts_id: ChartId,
    pub chart_of_accounts_omnibus_parent_code: AccountCode,
    pub chart_of_accounts_individual_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_government_entity_deposit_accounts_parent_code: AccountCode,
    pub chart_of_account_private_company_deposit_accounts_parent_code: AccountCode,
    pub chart_of_account_bank_deposit_accounts_parent_code: AccountCode,
    pub chart_of_account_financial_institution_deposit_accounts_parent_code: AccountCode,
    pub chart_of_account_non_domiciled_individual_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_frozen_individual_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code: AccountCode,
    pub chart_of_account_frozen_private_company_deposit_accounts_parent_code: AccountCode,
    pub chart_of_account_frozen_bank_deposit_accounts_parent_code: AccountCode,
    pub chart_of_account_frozen_financial_institution_deposit_accounts_parent_code: AccountCode,
    pub chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code: AccountCode,
}
