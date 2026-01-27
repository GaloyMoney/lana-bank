use serde::{Deserialize, Serialize};

use core_accounting::{AccountCode, AccountingBaseConfig, CalaAccountSetId, Chart, ChartId};
use domain_config::define_internal_config;

use super::error::ChartOfAccountsIntegrationError;
use crate::ledger::{
    LongTermDisbursedIntegrationMeta, LongTermInterestIntegrationMeta,
    OverdueDisbursedIntegrationMeta, ShortTermDisbursedIntegrationMeta,
    ShortTermInterestIntegrationMeta,
};

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

define_internal_config! {
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct ResolvedChartOfAccountsIntegrationConfig {
        pub config: ChartOfAccountsIntegrationConfig,

        pub facility_omnibus_parent_account_set_id: CalaAccountSetId,
        pub collateral_omnibus_parent_account_set_id: CalaAccountSetId,
        pub liquidation_proceeds_omnibus_parent_account_set_id: CalaAccountSetId,
        pub payments_made_omnibus_parent_account_set_id: CalaAccountSetId,
        pub interest_added_to_obligations_omnibus_parent_account_set_id: CalaAccountSetId,

        pub facility_parent_account_set_id: CalaAccountSetId,
        pub collateral_parent_account_set_id: CalaAccountSetId,
        pub collateral_in_liquidation_parent_account_set_id: CalaAccountSetId,
        pub interest_income_parent_account_set_id: CalaAccountSetId,
        pub fee_income_parent_account_set_id: CalaAccountSetId,
        pub payment_holding_parent_account_set_id: CalaAccountSetId,
        pub uncovered_outstanding_parent_account_set_id: CalaAccountSetId,

        pub short_term_disbursed_integration_meta: ShortTermDisbursedIntegrationMeta,
        pub long_term_disbursed_integration_meta: LongTermDisbursedIntegrationMeta,
        pub short_term_interest_integration_meta: ShortTermInterestIntegrationMeta,
        pub long_term_interest_integration_meta: LongTermInterestIntegrationMeta,
        pub overdue_disbursed_integration_meta: OverdueDisbursedIntegrationMeta,
    }

    spec {
        key: "credit-chart-of-accounts-integration";
    }
}

impl ResolvedChartOfAccountsIntegrationConfig {
    pub fn try_new(
        config: ChartOfAccountsIntegrationConfig,
        chart: &Chart,
        accounting_base_config: &AccountingBaseConfig,
    ) -> Result<Self, ChartOfAccountsIntegrationError> {
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
                &config.chart_of_account_facility_omnibus_parent_code,
            )?;
        let collateral_omnibus_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                &config.chart_of_account_collateral_omnibus_parent_code,
            )?;
        let liquidation_proceeds_omnibus_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                &config.chart_of_account_liquidation_proceeds_omnibus_parent_code,
            )?;
        let payments_made_omnibus_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                &config.chart_of_account_payments_made_omnibus_parent_code,
            )?;
        let interest_added_to_obligations_omnibus_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                &config.chart_of_account_interest_added_to_obligations_omnibus_parent_code,
            )?;

        let facility_parent_account_set_id = off_balance_sheet_account_set_member_parent_id(
            &config.chart_of_account_facility_parent_code,
        )?;
        let collateral_parent_account_set_id = off_balance_sheet_account_set_member_parent_id(
            &config.chart_of_account_collateral_parent_code,
        )?;
        let collateral_in_liquidation_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                &config.chart_of_account_collateral_in_liquidation_parent_code,
            )?;

        let interest_income_parent_account_set_id = revenue_account_set_member_parent_id(
            &config.chart_of_account_interest_income_parent_code,
        )?;
        let fee_income_parent_account_set_id =
            revenue_account_set_member_parent_id(&config.chart_of_account_fee_income_parent_code)?;
        let payment_holding_parent_account_set_id = asset_account_set_member_parent_id(
            &config.chart_of_account_payment_holding_parent_code,
        )?;
        let uncovered_outstanding_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                &config.chart_of_account_uncovered_outstanding_parent_code,
            )?;

        let short_term_disbursed_integration_meta = ShortTermDisbursedIntegrationMeta {
            short_term_individual_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config.chart_of_account_short_term_individual_disbursed_receivable_parent_code,
                )?,
            short_term_government_entity_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_short_term_government_entity_disbursed_receivable_parent_code,
                )?,
            short_term_private_company_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_short_term_private_company_disbursed_receivable_parent_code,
                )?,
            short_term_bank_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config.chart_of_account_short_term_bank_disbursed_receivable_parent_code,
                )?,
            short_term_financial_institution_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code,
                )?,
            short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
                )?,
            short_term_non_domiciled_company_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code,
                )?,
        };

        let long_term_disbursed_integration_meta = LongTermDisbursedIntegrationMeta {
            long_term_individual_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config.chart_of_account_long_term_individual_disbursed_receivable_parent_code,
                )?,
            long_term_government_entity_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_long_term_government_entity_disbursed_receivable_parent_code,
                )?,
            long_term_private_company_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_long_term_private_company_disbursed_receivable_parent_code,
                )?,
            long_term_bank_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config.chart_of_account_long_term_bank_disbursed_receivable_parent_code,
                )?,
            long_term_financial_institution_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code,
                )?,
            long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
                )?,
            long_term_non_domiciled_company_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code,
                )?,
        };

        let short_term_interest_integration_meta = ShortTermInterestIntegrationMeta {
            short_term_individual_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config.chart_of_account_short_term_individual_interest_receivable_parent_code,
                )?,
            short_term_government_entity_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_short_term_government_entity_interest_receivable_parent_code,
                )?,
            short_term_private_company_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_short_term_private_company_interest_receivable_parent_code,
                )?,
            short_term_bank_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config.chart_of_account_short_term_bank_interest_receivable_parent_code,
                )?,
            short_term_financial_institution_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_short_term_financial_institution_interest_receivable_parent_code,
                )?,
            short_term_foreign_agency_or_subsidiary_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
                )?,
            short_term_non_domiciled_company_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code,
                )?,
        };

        let long_term_interest_integration_meta = LongTermInterestIntegrationMeta {
            long_term_individual_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config.chart_of_account_long_term_individual_interest_receivable_parent_code,
                )?,
            long_term_government_entity_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_long_term_government_entity_interest_receivable_parent_code,
                )?,
            long_term_private_company_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_long_term_private_company_interest_receivable_parent_code,
                )?,
            long_term_bank_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config.chart_of_account_long_term_bank_interest_receivable_parent_code,
                )?,
            long_term_financial_institution_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_long_term_financial_institution_interest_receivable_parent_code,
                )?,
            long_term_foreign_agency_or_subsidiary_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
                )?,
            long_term_non_domiciled_company_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code,
                )?,
        };

        let overdue_disbursed_integration_meta = OverdueDisbursedIntegrationMeta {
            overdue_individual_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config.chart_of_account_overdue_individual_disbursed_receivable_parent_code,
                )?,
            overdue_government_entity_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_overdue_government_entity_disbursed_receivable_parent_code,
                )?,
            overdue_private_company_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config.chart_of_account_overdue_private_company_disbursed_receivable_parent_code,
                )?,
            overdue_bank_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config.chart_of_account_overdue_bank_disbursed_receivable_parent_code,
                )?,
            overdue_financial_institution_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code,
                )?,
            overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
                )?,
            overdue_non_domiciled_company_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    &config
                        .chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code,
                )?,
        };

        Ok(Self {
            config,

            facility_omnibus_parent_account_set_id,
            collateral_omnibus_parent_account_set_id,
            liquidation_proceeds_omnibus_parent_account_set_id,
            payments_made_omnibus_parent_account_set_id,
            interest_added_to_obligations_omnibus_parent_account_set_id,

            facility_parent_account_set_id,
            collateral_parent_account_set_id,
            collateral_in_liquidation_parent_account_set_id,
            interest_income_parent_account_set_id,
            fee_income_parent_account_set_id,
            payment_holding_parent_account_set_id,
            uncovered_outstanding_parent_account_set_id,

            short_term_disbursed_integration_meta,
            long_term_disbursed_integration_meta,
            short_term_interest_integration_meta,
            long_term_interest_integration_meta,
            overdue_disbursed_integration_meta,
        })
    }
}
