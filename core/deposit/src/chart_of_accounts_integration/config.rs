use serde::{Deserialize, Serialize};

use core_accounting::{AccountCategory, AccountCode, CalaAccountSetId, Chart, ChartId};
use domain_config::define_internal_config;

use super::error::ChartOfAccountsIntegrationError;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
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

define_internal_config! {
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub(crate) struct ResolvedChartOfAccountsIntegrationConfig {
        pub(crate) config: ChartOfAccountsIntegrationConfig,

        pub(crate) omnibus_parent_account_set_id: CalaAccountSetId,

        pub(crate) individual_deposit_accounts_parent_account_set_id: CalaAccountSetId,
        pub(crate) government_entity_deposit_accounts_parent_account_set_id: CalaAccountSetId,
        pub(crate) private_company_deposit_accounts_parent_account_set_id: CalaAccountSetId,
        pub(crate) bank_deposit_accounts_parent_account_set_id: CalaAccountSetId,
        pub(crate) financial_institution_deposit_accounts_parent_account_set_id: CalaAccountSetId,
        pub(crate) non_domiciled_individual_deposit_accounts_parent_account_set_id: CalaAccountSetId,

        pub(crate) frozen_individual_deposit_accounts_parent_account_set_id: CalaAccountSetId,
        pub(crate) frozen_government_entity_deposit_accounts_parent_account_set_id: CalaAccountSetId,
        pub(crate) frozen_private_company_deposit_accounts_parent_account_set_id: CalaAccountSetId,
        pub(crate) frozen_bank_deposit_accounts_parent_account_set_id: CalaAccountSetId,
        pub(crate) frozen_financial_institution_deposit_accounts_parent_account_set_id:
            CalaAccountSetId,
        pub(crate) frozen_non_domiciled_individual_deposit_accounts_parent_account_set_id:
            CalaAccountSetId,
    }

    spec {
        key: "deposit-chart-of-accounts-integration";
    }
}

impl ResolvedChartOfAccountsIntegrationConfig {
    pub(super) fn try_new(
        config: ChartOfAccountsIntegrationConfig,
        chart: &Chart,
    ) -> Result<Self, ChartOfAccountsIntegrationError> {
        let asset_account_set_member_parent_id =
            |code: &AccountCode| -> Result<CalaAccountSetId, ChartOfAccountsIntegrationError> {
                chart
                    .find_account_set_id_in_category(code, AccountCategory::Asset)
                    .ok_or_else(|| {
                        core_accounting::chart_of_accounts::error::ChartOfAccountsError::InvalidAccountCategory {
                            code: code.clone(),
                            category: AccountCategory::Asset,
                        }
                        .into()
                    })
            };
        let liabilities_account_set_member_parent_id =
            |code: &AccountCode| -> Result<CalaAccountSetId, ChartOfAccountsIntegrationError> {
                chart
                    .find_account_set_id_in_category(code, AccountCategory::Liability)
                    .ok_or_else(|| {
                        core_accounting::chart_of_accounts::error::ChartOfAccountsError::InvalidAccountCategory {
                            code: code.clone(),
                            category: AccountCategory::Liability,
                        }
                        .into()
                    })
            };

        let individual_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &config.chart_of_accounts_individual_deposit_accounts_parent_code,
            )?;

        let government_entity_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &config.chart_of_accounts_government_entity_deposit_accounts_parent_code,
            )?;

        let private_company_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &config.chart_of_account_private_company_deposit_accounts_parent_code,
            )?;

        let bank_deposit_accounts_parent_account_set_id = liabilities_account_set_member_parent_id(
            &config.chart_of_account_bank_deposit_accounts_parent_code,
        )?;

        let financial_institution_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &config.chart_of_account_financial_institution_deposit_accounts_parent_code,
            )?;

        let non_domiciled_individual_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &config.chart_of_account_non_domiciled_individual_deposit_accounts_parent_code,
            )?;

        let frozen_individual_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &config.chart_of_accounts_frozen_individual_deposit_accounts_parent_code,
            )?;

        let frozen_government_entity_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &config.chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code,
            )?;

        let frozen_private_company_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &config.chart_of_account_frozen_private_company_deposit_accounts_parent_code,
            )?;

        let frozen_bank_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &config.chart_of_account_frozen_bank_deposit_accounts_parent_code,
            )?;

        let frozen_financial_institution_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &config.chart_of_account_frozen_financial_institution_deposit_accounts_parent_code,
            )?;

        let frozen_non_domiciled_individual_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &config
                    .chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code,
            )?;

        let omnibus_parent_account_set_id =
            asset_account_set_member_parent_id(&config.chart_of_accounts_omnibus_parent_code)?;
        Ok(Self {
            config,

            omnibus_parent_account_set_id,
            individual_deposit_accounts_parent_account_set_id,
            government_entity_deposit_accounts_parent_account_set_id,
            private_company_deposit_accounts_parent_account_set_id,
            bank_deposit_accounts_parent_account_set_id,
            financial_institution_deposit_accounts_parent_account_set_id,
            non_domiciled_individual_deposit_accounts_parent_account_set_id,
            frozen_individual_deposit_accounts_parent_account_set_id,
            frozen_government_entity_deposit_accounts_parent_account_set_id,
            frozen_private_company_deposit_accounts_parent_account_set_id,
            frozen_bank_deposit_accounts_parent_account_set_id,
            frozen_financial_institution_deposit_accounts_parent_account_set_id,
            frozen_non_domiciled_individual_deposit_accounts_parent_account_set_id,
        })
    }
}
