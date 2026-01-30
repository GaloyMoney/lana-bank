use std::{fs, path::PathBuf};

use serde::Deserialize;

use crate::{
    accounting::Chart,
    accounting_init::AccountingInitError,
    deposit::{ChartOfAccountsIntegrationConfig, Deposits},
};

use rbac_types::Subject;

#[derive(Deserialize)]
struct DepositConfigData {
    omnibus_parent_code: String,
    individual_deposit_accounts_parent_code: String,
    government_entity_deposit_accounts_parent_code: String,
    private_company_deposit_accounts_parent_code: String,
    bank_deposit_accounts_parent_code: String,
    financial_institution_deposit_accounts_parent_code: String,
    non_domiciled_individual_deposit_accounts_parent_code: String,
    frozen_individual_deposit_accounts_parent_code: String,
    frozen_government_entity_deposit_accounts_parent_code: String,
    frozen_private_company_deposit_accounts_parent_code: String,
    frozen_bank_deposit_accounts_parent_code: String,
    frozen_financial_institution_deposit_accounts_parent_code: String,
    frozen_non_domiciled_individual_deposit_accounts_parent_code: String,
}

pub(in crate::accounting_init::seed) async fn deposit_module_configure(
    deposit: &Deposits,
    chart: &Chart,
    config_path: PathBuf,
) -> Result<(), AccountingInitError> {
    let data = fs::read_to_string(config_path)?;
    let DepositConfigData {
        omnibus_parent_code,
        individual_deposit_accounts_parent_code,
        government_entity_deposit_accounts_parent_code,
        private_company_deposit_accounts_parent_code,
        bank_deposit_accounts_parent_code,
        financial_institution_deposit_accounts_parent_code,
        non_domiciled_individual_deposit_accounts_parent_code,
        frozen_individual_deposit_accounts_parent_code,
        frozen_government_entity_deposit_accounts_parent_code,
        frozen_private_company_deposit_accounts_parent_code,
        frozen_bank_deposit_accounts_parent_code,
        frozen_financial_institution_deposit_accounts_parent_code,
        frozen_non_domiciled_individual_deposit_accounts_parent_code,
    } = serde_json::from_str(&data)?;

    let config_values = ChartOfAccountsIntegrationConfig {
        chart_of_accounts_id: chart.id,
        chart_of_accounts_omnibus_parent_code: omnibus_parent_code.parse()?,
        chart_of_accounts_individual_deposit_accounts_parent_code:
            individual_deposit_accounts_parent_code.parse()?,
        chart_of_accounts_government_entity_deposit_accounts_parent_code:
            government_entity_deposit_accounts_parent_code.parse()?,
        chart_of_account_private_company_deposit_accounts_parent_code:
            private_company_deposit_accounts_parent_code.parse()?,
        chart_of_account_bank_deposit_accounts_parent_code: bank_deposit_accounts_parent_code
            .parse()?,
        chart_of_account_financial_institution_deposit_accounts_parent_code:
            financial_institution_deposit_accounts_parent_code.parse()?,
        chart_of_account_non_domiciled_individual_deposit_accounts_parent_code:
            non_domiciled_individual_deposit_accounts_parent_code.parse()?,
        chart_of_accounts_frozen_individual_deposit_accounts_parent_code:
            frozen_individual_deposit_accounts_parent_code.parse()?,
        chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code:
            frozen_government_entity_deposit_accounts_parent_code.parse()?,
        chart_of_account_frozen_private_company_deposit_accounts_parent_code:
            frozen_private_company_deposit_accounts_parent_code.parse()?,
        chart_of_account_frozen_bank_deposit_accounts_parent_code:
            frozen_bank_deposit_accounts_parent_code.parse()?,
        chart_of_account_frozen_financial_institution_deposit_accounts_parent_code:
            frozen_financial_institution_deposit_accounts_parent_code.parse()?,
        chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code:
            frozen_non_domiciled_individual_deposit_accounts_parent_code.parse()?,
    };

    match deposit
        .chart_of_accounts_integrations()
        .set_config(&Subject::System, chart, config_values)
        .await
    {
        Ok(_) => (),
        Err(core_deposit::ChartOfAccountsIntegrationError::ConfigAlreadyExists) => (),
        Err(e) => return Err(e.into()),
    };

    Ok(())
}
