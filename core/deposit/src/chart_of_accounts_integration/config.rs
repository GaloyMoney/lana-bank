use money::CurrencyCode;
use serde::{Deserialize, Serialize};

use chart_primitives::{AccountCode, CalaAccountSetId, ChartId, ChartLookup, ChartLookupError};
use domain_config::define_internal_config;

use super::error::ChartOfAccountsIntegrationError;
use crate::primitives::{
    DepositAccountType,
    account_sets::{DEPOSIT_ACCOUNT_SET_CATALOG, DepositAccountCategory},
};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct ChartOfAccountsIntegrationConfig {
    pub chart_of_accounts_id: ChartId,
    pub chart_of_accounts_omnibus_parent_code: AccountCode,
    pub chart_of_accounts_individual_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_government_entity_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_private_company_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_bank_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_financial_institution_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_non_domiciled_company_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_frozen_individual_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_frozen_private_company_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_frozen_bank_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_frozen_financial_institution_deposit_accounts_parent_code: AccountCode,
    pub chart_of_accounts_frozen_non_domiciled_company_deposit_accounts_parent_code: AccountCode,
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
        pub(crate) non_domiciled_company_deposit_accounts_parent_account_set_id: CalaAccountSetId,

        pub(crate) frozen_individual_deposit_accounts_parent_account_set_id: CalaAccountSetId,
        pub(crate) frozen_government_entity_deposit_accounts_parent_account_set_id: CalaAccountSetId,
        pub(crate) frozen_private_company_deposit_accounts_parent_account_set_id: CalaAccountSetId,
        pub(crate) frozen_bank_deposit_accounts_parent_account_set_id: CalaAccountSetId,
        pub(crate) frozen_financial_institution_deposit_accounts_parent_account_set_id:
            CalaAccountSetId,
        pub(crate) frozen_non_domiciled_company_deposit_accounts_parent_account_set_id:
            CalaAccountSetId,
    }

    spec {
        key: "deposit-chart-of-accounts-integration";
    }
}

impl ResolvedChartOfAccountsIntegrationConfig {
    pub(super) fn try_new(
        config: ChartOfAccountsIntegrationConfig,
        chart: &dyn ChartLookup,
    ) -> Result<Self, ChartOfAccountsIntegrationError> {
        let category_account_set_member_parent_id =
            |code: &AccountCode,
             category: DepositAccountCategory|
             -> Result<CalaAccountSetId, ChartOfAccountsIntegrationError> {
                chart
                    .find_account_set_id_in_category(code, category.into())
                    .ok_or_else(|| {
                        ChartLookupError::InvalidAccountCategory {
                            code: code.clone(),
                            category: category.into(),
                        }
                        .into()
                    })
            };

        let catalog = &DEPOSIT_ACCOUNT_SET_CATALOG;
        let deposit = catalog.deposit();
        let frozen = catalog.frozen();
        let omnibus_account =
            |currency: CurrencyCode| -> Result<_, ChartOfAccountsIntegrationError> {
                catalog.find_omnibus(currency).ok_or(
                    ChartOfAccountsIntegrationError::MissingOmnibusAccountSetSpec { currency },
                )
            };
        let deposit_account_category =
            |account_type: DepositAccountType,
             currency: CurrencyCode|
             -> Result<DepositAccountCategory, ChartOfAccountsIntegrationError> {
                deposit
                    .find(account_type, currency)
                    .map(|spec| spec.account_category)
                    .ok_or(ChartOfAccountsIntegrationError::MissingAccountSetSpec {
                        account_type,
                        currency,
                    })
            };
        let frozen_account_category =
            |account_type: DepositAccountType,
             currency: CurrencyCode|
             -> Result<DepositAccountCategory, ChartOfAccountsIntegrationError> {
                frozen
                    .find(account_type, currency)
                    .map(|spec| spec.account_category)
                    .ok_or(ChartOfAccountsIntegrationError::MissingAccountSetSpec {
                        account_type,
                        currency,
                    })
            };

        let omnibus_parent_account_set_id = category_account_set_member_parent_id(
            &config.chart_of_accounts_omnibus_parent_code,
            omnibus_account(CurrencyCode::USD)?.account_category,
        )?;

        let individual_deposit_accounts_parent_account_set_id =
            category_account_set_member_parent_id(
                &config.chart_of_accounts_individual_deposit_accounts_parent_code,
                deposit_account_category(DepositAccountType::Individual, CurrencyCode::USD)?,
            )?;

        let government_entity_deposit_accounts_parent_account_set_id =
            category_account_set_member_parent_id(
                &config.chart_of_accounts_government_entity_deposit_accounts_parent_code,
                deposit_account_category(DepositAccountType::GovernmentEntity, CurrencyCode::USD)?,
            )?;

        let private_company_deposit_accounts_parent_account_set_id =
            category_account_set_member_parent_id(
                &config.chart_of_accounts_private_company_deposit_accounts_parent_code,
                deposit_account_category(DepositAccountType::PrivateCompany, CurrencyCode::USD)?,
            )?;

        let bank_deposit_accounts_parent_account_set_id = category_account_set_member_parent_id(
            &config.chart_of_accounts_bank_deposit_accounts_parent_code,
            deposit_account_category(DepositAccountType::Bank, CurrencyCode::USD)?,
        )?;

        let financial_institution_deposit_accounts_parent_account_set_id =
            category_account_set_member_parent_id(
                &config.chart_of_accounts_financial_institution_deposit_accounts_parent_code,
                deposit_account_category(
                    DepositAccountType::FinancialInstitution,
                    CurrencyCode::USD,
                )?,
            )?;

        let non_domiciled_company_deposit_accounts_parent_account_set_id =
            category_account_set_member_parent_id(
                &config.chart_of_accounts_non_domiciled_company_deposit_accounts_parent_code,
                deposit_account_category(
                    DepositAccountType::NonDomiciledCompany,
                    CurrencyCode::USD,
                )?,
            )?;

        let frozen_individual_deposit_accounts_parent_account_set_id =
            category_account_set_member_parent_id(
                &config.chart_of_accounts_frozen_individual_deposit_accounts_parent_code,
                frozen_account_category(DepositAccountType::Individual, CurrencyCode::USD)?,
            )?;

        let frozen_government_entity_deposit_accounts_parent_account_set_id =
            category_account_set_member_parent_id(
                &config.chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code,
                frozen_account_category(DepositAccountType::GovernmentEntity, CurrencyCode::USD)?,
            )?;

        let frozen_private_company_deposit_accounts_parent_account_set_id =
            category_account_set_member_parent_id(
                &config.chart_of_accounts_frozen_private_company_deposit_accounts_parent_code,
                frozen_account_category(DepositAccountType::PrivateCompany, CurrencyCode::USD)?,
            )?;

        let frozen_bank_deposit_accounts_parent_account_set_id =
            category_account_set_member_parent_id(
                &config.chart_of_accounts_frozen_bank_deposit_accounts_parent_code,
                frozen_account_category(DepositAccountType::Bank, CurrencyCode::USD)?,
            )?;

        let frozen_financial_institution_deposit_accounts_parent_account_set_id =
            category_account_set_member_parent_id(
                &config.chart_of_accounts_frozen_financial_institution_deposit_accounts_parent_code,
                frozen_account_category(
                    DepositAccountType::FinancialInstitution,
                    CurrencyCode::USD,
                )?,
            )?;

        let frozen_non_domiciled_company_deposit_accounts_parent_account_set_id =
            category_account_set_member_parent_id(
                &config.chart_of_accounts_frozen_non_domiciled_company_deposit_accounts_parent_code,
                frozen_account_category(
                    DepositAccountType::NonDomiciledCompany,
                    CurrencyCode::USD,
                )?,
            )?;

        Ok(Self {
            config,

            omnibus_parent_account_set_id,
            individual_deposit_accounts_parent_account_set_id,
            government_entity_deposit_accounts_parent_account_set_id,
            private_company_deposit_accounts_parent_account_set_id,
            bank_deposit_accounts_parent_account_set_id,
            financial_institution_deposit_accounts_parent_account_set_id,
            non_domiciled_company_deposit_accounts_parent_account_set_id,
            frozen_individual_deposit_accounts_parent_account_set_id,
            frozen_government_entity_deposit_accounts_parent_account_set_id,
            frozen_private_company_deposit_accounts_parent_account_set_id,
            frozen_bank_deposit_accounts_parent_account_set_id,
            frozen_financial_institution_deposit_accounts_parent_account_set_id,
            frozen_non_domiciled_company_deposit_accounts_parent_account_set_id,
        })
    }
}
