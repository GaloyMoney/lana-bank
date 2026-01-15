use serde::{Deserialize, Serialize};

use audit::AuditInfo;
use core_accounting::{AccountCode, CalaAccountSetId, Chart, ChartId};

use crate::{error::CoreDepositError, ledger::ChartOfAccountsIntegrationMeta};

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

impl ChartOfAccountsIntegrationConfig {
    pub(super) fn try_into_meta(
        &self,
        chart: &Chart,
        audit_info: AuditInfo,
    ) -> Result<ChartOfAccountsIntegrationMeta, CoreDepositError> {
        let accounting_base_config = chart
            .find_accounting_base_config()
            .ok_or(CoreDepositError::AccountingBaseConfigNotFound)?;

        let asset_account_set_member_parent_id =
            |code: &AccountCode| -> Result<CalaAccountSetId, CoreDepositError> {
                let id = chart.account_set_id_from_code(code)?;
                if !accounting_base_config.is_assets_account_set_member(code) {
                    return Err(CoreDepositError::InvalidAccountingAccountSetParent(
                        code.to_string(),
                    ));
                }
                Ok(id)
            };

        let liabilities_account_set_member_parent_id =
            |code: &AccountCode| -> Result<CalaAccountSetId, CoreDepositError> {
                let id = chart.account_set_id_from_code(code)?;
                if !accounting_base_config.is_liabilities_account_set_member(code) {
                    return Err(CoreDepositError::InvalidAccountingAccountSetParent(
                        code.to_string(),
                    ));
                }
                Ok(id)
            };

        let omnibus_parent_account_set_id =
            asset_account_set_member_parent_id(&self.chart_of_accounts_omnibus_parent_code)?;

        let individual_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &self.chart_of_accounts_individual_deposit_accounts_parent_code,
            )?;

        let government_entity_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &self.chart_of_accounts_government_entity_deposit_accounts_parent_code,
            )?;

        let private_company_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &self.chart_of_account_private_company_deposit_accounts_parent_code,
            )?;

        let bank_deposit_accounts_parent_account_set_id = liabilities_account_set_member_parent_id(
            &self.chart_of_account_bank_deposit_accounts_parent_code,
        )?;

        let financial_institution_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &self.chart_of_account_financial_institution_deposit_accounts_parent_code,
            )?;

        let non_domiciled_individual_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &self.chart_of_account_non_domiciled_individual_deposit_accounts_parent_code,
            )?;

        let frozen_individual_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &self.chart_of_accounts_frozen_individual_deposit_accounts_parent_code,
            )?;

        let frozen_government_entity_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &self.chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code,
            )?;

        let frozen_private_company_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &self.chart_of_account_frozen_private_company_deposit_accounts_parent_code,
            )?;

        let frozen_bank_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &self.chart_of_account_frozen_bank_deposit_accounts_parent_code,
            )?;

        let frozen_financial_institution_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &self.chart_of_account_frozen_financial_institution_deposit_accounts_parent_code,
            )?;

        let frozen_non_domiciled_individual_deposit_accounts_parent_account_set_id =
            liabilities_account_set_member_parent_id(
                &self.chart_of_account_frozen_non_domiciled_individual_deposit_accounts_parent_code,
            )?;

        Ok(ChartOfAccountsIntegrationMeta {
            config: self.clone(),
            audit_info,
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
