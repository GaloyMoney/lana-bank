pub mod error;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use audit::{AuditInfo, AuditSvc};
use authz::PermissionCheck;
use core_accounting::{AccountCode, AccountingBaseConfig, CalaAccountSetId, Chart, ChartId};

use crate::{CoreCreditAction, CoreCreditObject, ledger::*};

use error::ChartOfAccountsIntegrationError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChartOfAccountsIntegrationConfig {
    pub chart_of_accounts_id: ChartId,
    pub chart_of_account_facility_omnibus_parent_code: AccountCode,
    pub chart_of_account_collateral_omnibus_parent_code: AccountCode,
    pub chart_of_account_liquidation_proceeds_omnibus_parent_code: AccountCode,
    pub chart_of_account_payments_made_omnibus_parent_code: AccountCode,
    pub chart_of_account_interest_added_to_obligations_omnibus_parent_code: AccountCode,
    pub chart_of_account_facility_parent_code: AccountCode,
    pub chart_of_account_collateral_parent_code: AccountCode,
    pub chart_of_account_collateral_in_liquidation_parent_code: AccountCode,
    pub chart_of_account_interest_income_parent_code: AccountCode,
    pub chart_of_account_fee_income_parent_code: AccountCode,
    pub chart_of_account_payment_holding_parent_code: AccountCode,
    pub chart_of_account_uncovered_outstanding_parent_code: AccountCode,

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
    pub(super) fn validate_and_transform(
        &self,
        chart: &Chart,
        audit_info: AuditInfo,
        accounting_base_config: &AccountingBaseConfig,
    ) -> Result<ChartOfAccountsIntegrationMeta, ChartOfAccountsIntegrationError> {
        let off_balance_sheet_account_set_member_parent_id =
            |code: &AccountCode| -> Result<CalaAccountSetId, ChartOfAccountsIntegrationError> {
                let id = chart.account_set_id_from_code(code)?;
                if !accounting_base_config.is_off_balance_sheet_account_set_or_account(code) {
                    return Err(
                        ChartOfAccountsIntegrationError::InvalidAccountingAccountSetParent(
                            code.to_string(),
                        ),
                    );
                }
                Ok(id)
            };

        let revenue_account_set_member_parent_id =
            |code: &AccountCode| -> Result<CalaAccountSetId, ChartOfAccountsIntegrationError> {
                let id = chart.account_set_id_from_code(code)?;
                if !accounting_base_config.is_revenue_account_set_or_account(code) {
                    return Err(
                        ChartOfAccountsIntegrationError::InvalidAccountingAccountSetParent(
                            code.to_string(),
                        ),
                    );
                }
                Ok(id)
            };

        let asset_account_set_member_parent_id =
            |code: &AccountCode| -> Result<CalaAccountSetId, ChartOfAccountsIntegrationError> {
                let id = chart.account_set_id_from_code(code)?;
                if !accounting_base_config.is_assets_account_set_or_account(code) {
                    return Err(
                        ChartOfAccountsIntegrationError::InvalidAccountingAccountSetParent(
                            code.to_string(),
                        ),
                    );
                }
                Ok(id)
            };

        let facility_omnibus_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                &self.chart_of_account_facility_omnibus_parent_code,
            )?;
        let collateral_omnibus_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                &self.chart_of_account_collateral_omnibus_parent_code,
            )?;
        let liquidation_proceeds_omnibus_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                &self.chart_of_account_liquidation_proceeds_omnibus_parent_code,
            )?;
        let facility_parent_account_set_id = off_balance_sheet_account_set_member_parent_id(
            &self.chart_of_account_facility_parent_code,
        )?;
        let collateral_parent_account_set_id = off_balance_sheet_account_set_member_parent_id(
            &self.chart_of_account_collateral_parent_code,
        )?;
        let collateral_in_liquidation_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                &self.chart_of_account_collateral_in_liquidation_parent_code,
            )?;

        let interest_income_parent_account_set_id = revenue_account_set_member_parent_id(
            &self.chart_of_account_interest_income_parent_code,
        )?;
        let fee_income_parent_account_set_id =
            revenue_account_set_member_parent_id(&self.chart_of_account_fee_income_parent_code)?;
        let payment_holding_parent_account_set_id =
            asset_account_set_member_parent_id(&self.chart_of_account_payment_holding_parent_code)?;

        let short_term_disbursed_integration_meta = ShortTermDisbursedIntegrationMeta {
            short_term_individual_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_short_term_individual_disbursed_receivable_parent_code)?,
            short_term_government_entity_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_short_term_government_entity_disbursed_receivable_parent_code)?,
            short_term_private_company_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_short_term_private_company_disbursed_receivable_parent_code)?,
            short_term_bank_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_short_term_bank_disbursed_receivable_parent_code)?,
            short_term_financial_institution_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code)?,
            short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code)?,
            short_term_non_domiciled_company_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code)?,
        };

        let long_term_disbursed_integration_meta = LongTermDisbursedIntegrationMeta {
            long_term_individual_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_long_term_individual_disbursed_receivable_parent_code)?,
            long_term_government_entity_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_long_term_government_entity_disbursed_receivable_parent_code)?,
            long_term_private_company_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_long_term_private_company_disbursed_receivable_parent_code)?,
            long_term_bank_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_long_term_bank_disbursed_receivable_parent_code)?,
            long_term_financial_institution_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code)?,
            long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code)?,
            long_term_non_domiciled_company_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code)?,
        };

        let short_term_interest_integration_meta = ShortTermInterestIntegrationMeta {
            short_term_individual_interest_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_short_term_individual_interest_receivable_parent_code)?,
            short_term_government_entity_interest_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_short_term_government_entity_interest_receivable_parent_code)?,
            short_term_private_company_interest_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_short_term_private_company_interest_receivable_parent_code)?,
            short_term_bank_interest_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_short_term_bank_interest_receivable_parent_code)?,
            short_term_financial_institution_interest_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_short_term_financial_institution_interest_receivable_parent_code)?,
            short_term_foreign_agency_or_subsidiary_interest_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code)?,
            short_term_non_domiciled_company_interest_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code)?,
        };

        let long_term_interest_integration_meta = LongTermInterestIntegrationMeta {
            long_term_individual_interest_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_long_term_individual_interest_receivable_parent_code)?,
            long_term_government_entity_interest_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_long_term_government_entity_interest_receivable_parent_code)?,
            long_term_private_company_interest_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_long_term_private_company_interest_receivable_parent_code)?,
            long_term_bank_interest_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_long_term_bank_interest_receivable_parent_code)?,
            long_term_financial_institution_interest_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_long_term_financial_institution_interest_receivable_parent_code)?,
            long_term_foreign_agency_or_subsidiary_interest_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code)?,
            long_term_non_domiciled_company_interest_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code)?,
        };

        let overdue_disbursed_integration_meta = OverdueDisbursedIntegrationMeta {
            overdue_individual_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_overdue_individual_disbursed_receivable_parent_code)?,
            overdue_government_entity_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_overdue_government_entity_disbursed_receivable_parent_code)?,
            overdue_private_company_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_overdue_private_company_disbursed_receivable_parent_code)?,
            overdue_bank_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_overdue_bank_disbursed_receivable_parent_code)?,
            overdue_financial_institution_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code)?,
            overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code)?,
            overdue_non_domiciled_company_disbursed_receivable_parent_account_set_id: asset_account_set_member_parent_id(&self.chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code)?,
        };

        Ok(ChartOfAccountsIntegrationMeta {
            config: self.clone(),
            audit_info,
            facility_omnibus_parent_account_set_id,
            collateral_omnibus_parent_account_set_id,
            liquidation_proceeds_omnibus_parent_account_set_id,
            facility_parent_account_set_id,
            collateral_parent_account_set_id,
            collateral_in_liquidation_parent_account_set_id,
            interest_income_parent_account_set_id,
            fee_income_parent_account_set_id,
            payment_holding_parent_account_set_id,
            short_term_disbursed_integration_meta,
            long_term_disbursed_integration_meta,
            short_term_interest_integration_meta,
            long_term_interest_integration_meta,
            overdue_disbursed_integration_meta,
        })
    }
}

pub struct ChartOfAccountsIntegrations<Perms>
where
    Perms: PermissionCheck,
{
    authz: Arc<Perms>,
    ledger: Arc<CreditLedger>,
}

impl<Perms> Clone for ChartOfAccountsIntegrations<Perms>
where
    Perms: PermissionCheck,
{
    fn clone(&self) -> Self {
        Self {
            authz: self.authz.clone(),
            ledger: self.ledger.clone(),
        }
    }
}

impl<Perms> ChartOfAccountsIntegrations<Perms>
where
    Perms: PermissionCheck,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Action: From<CoreCreditAction>,
    <<Perms as PermissionCheck>::Audit as AuditSvc>::Object: From<CoreCreditObject>,
{
    pub fn new(authz: Arc<Perms>, ledger: Arc<CreditLedger>) -> Self {
        Self { authz, ledger }
    }

    pub async fn set_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
        chart: &Chart,
        config: ChartOfAccountsIntegrationConfig,
    ) -> Result<ChartOfAccountsIntegrationConfig, ChartOfAccountsIntegrationError> {
        if chart.id != config.chart_of_accounts_id {
            return Err(ChartOfAccountsIntegrationError::ChartIdMismatch);
        }

        if self
            .ledger
            .get_chart_of_accounts_integration_config()
            .await?
            .is_some()
        {
            return Err(ChartOfAccountsIntegrationError::CreditConfigAlreadyExists);
        }
        let accounting_base_config = chart
            .accounting_base_config()
            .ok_or(ChartOfAccountsIntegrationError::AccountingBaseConfigNotFound)?;

        let audit_info = self
            .authz
            .enforce_permission(
                sub,
                CoreCreditObject::chart_of_accounts_integration(),
                CoreCreditAction::CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_UPDATE,
            )
            .await?;

        let charts_integration_meta =
            config.validate_and_transform(chart, audit_info, &accounting_base_config)?;
        self.ledger
            .attach_chart_of_accounts_account_sets(charts_integration_meta)
            .await?;

        Ok(config)
    }

    pub async fn get_config(
        &self,
        sub: &<<Perms as PermissionCheck>::Audit as AuditSvc>::Subject,
    ) -> Result<Option<ChartOfAccountsIntegrationConfig>, ChartOfAccountsIntegrationError> {
        self.authz
            .enforce_permission(
                sub,
                CoreCreditObject::chart_of_accounts_integration(),
                CoreCreditAction::CHART_OF_ACCOUNTS_INTEGRATION_CONFIG_READ,
            )
            .await?;
        Ok(self
            .ledger
            .get_chart_of_accounts_integration_config()
            .await?)
    }
}
