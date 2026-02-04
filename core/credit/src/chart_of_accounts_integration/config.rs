use serde::{Deserialize, Serialize};

use core_accounting::{AccountCategory, AccountCode, CalaAccountSetId, Chart, ChartId};
use domain_config::define_internal_config;

use super::error::ChartOfAccountsIntegrationError;
use crate::ledger::{
    LongTermDisbursedIntegrationMeta, LongTermInterestIntegrationMeta,
    OverdueDisbursedIntegrationMeta, ShortTermDisbursedIntegrationMeta,
    ShortTermInterestIntegrationMeta,
};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
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
    #[derive(Serialize, Deserialize, Clone)]
    pub(crate) struct ResolvedChartOfAccountsIntegrationConfig {
        pub(crate) config: ChartOfAccountsIntegrationConfig,

        pub(crate) facility_omnibus_parent_account_set_id: CalaAccountSetId,
        pub(crate) collateral_omnibus_parent_account_set_id: CalaAccountSetId,
        pub(crate) liquidation_proceeds_omnibus_parent_account_set_id: CalaAccountSetId,
        pub(crate) payments_made_omnibus_parent_account_set_id: CalaAccountSetId,
        pub(crate) interest_added_to_obligations_omnibus_parent_account_set_id: CalaAccountSetId,

        pub(crate) facility_parent_account_set_id: CalaAccountSetId,
        pub(crate) collateral_parent_account_set_id: CalaAccountSetId,
        pub(crate) collateral_in_liquidation_parent_account_set_id: CalaAccountSetId,
        pub(crate) interest_income_parent_account_set_id: CalaAccountSetId,
        pub(crate) fee_income_parent_account_set_id: CalaAccountSetId,
        pub(crate) payment_holding_parent_account_set_id: CalaAccountSetId,
        pub(crate) uncovered_outstanding_parent_account_set_id: CalaAccountSetId,

        pub(crate) short_term_disbursed_integration_meta: ShortTermDisbursedIntegrationMeta,
        pub(crate) long_term_disbursed_integration_meta: LongTermDisbursedIntegrationMeta,
        pub(crate) short_term_interest_integration_meta: ShortTermInterestIntegrationMeta,
        pub(crate) long_term_interest_integration_meta: LongTermInterestIntegrationMeta,
        pub(crate) overdue_disbursed_integration_meta: OverdueDisbursedIntegrationMeta,
    }

    spec {
        key: "credit-chart-of-accounts-integration";
    }
}

impl ResolvedChartOfAccountsIntegrationConfig {
    pub(super) fn try_new(
        config: ChartOfAccountsIntegrationConfig,
        chart: &Chart,
    ) -> Result<Self, ChartOfAccountsIntegrationError> {
        let ChartOfAccountsIntegrationConfig {
            chart_of_accounts_id: _,
            chart_of_account_facility_omnibus_parent_code,
            chart_of_account_collateral_omnibus_parent_code,
            chart_of_account_liquidation_proceeds_omnibus_parent_code,
            chart_of_account_payments_made_omnibus_parent_code,
            chart_of_account_interest_added_to_obligations_omnibus_parent_code,
            chart_of_account_facility_parent_code,
            chart_of_account_collateral_parent_code,
            chart_of_account_collateral_in_liquidation_parent_code,
            chart_of_account_interest_income_parent_code,
            chart_of_account_fee_income_parent_code,
            chart_of_account_payment_holding_parent_code,
            chart_of_account_uncovered_outstanding_parent_code,
            chart_of_account_short_term_individual_disbursed_receivable_parent_code,
            chart_of_account_short_term_government_entity_disbursed_receivable_parent_code,
            chart_of_account_short_term_private_company_disbursed_receivable_parent_code,
            chart_of_account_short_term_bank_disbursed_receivable_parent_code,
            chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code,
            chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
            chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code,
            chart_of_account_long_term_individual_disbursed_receivable_parent_code,
            chart_of_account_long_term_government_entity_disbursed_receivable_parent_code,
            chart_of_account_long_term_private_company_disbursed_receivable_parent_code,
            chart_of_account_long_term_bank_disbursed_receivable_parent_code,
            chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code,
            chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
            chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code,
            chart_of_account_short_term_individual_interest_receivable_parent_code,
            chart_of_account_short_term_government_entity_interest_receivable_parent_code,
            chart_of_account_short_term_private_company_interest_receivable_parent_code,
            chart_of_account_short_term_bank_interest_receivable_parent_code,
            chart_of_account_short_term_financial_institution_interest_receivable_parent_code,
            chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
            chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code,
            chart_of_account_long_term_individual_interest_receivable_parent_code,
            chart_of_account_long_term_government_entity_interest_receivable_parent_code,
            chart_of_account_long_term_private_company_interest_receivable_parent_code,
            chart_of_account_long_term_bank_interest_receivable_parent_code,
            chart_of_account_long_term_financial_institution_interest_receivable_parent_code,
            chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
            chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code,
            chart_of_account_overdue_individual_disbursed_receivable_parent_code,
            chart_of_account_overdue_government_entity_disbursed_receivable_parent_code,
            chart_of_account_overdue_private_company_disbursed_receivable_parent_code,
            chart_of_account_overdue_bank_disbursed_receivable_parent_code,
            chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code,
            chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
            chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code,
        } = &config;

        let off_balance_sheet_account_set_member_parent_id = |code: &AccountCode| -> Result<
            CalaAccountSetId,
            ChartOfAccountsIntegrationError,
        > {
            chart
                    .find_account_set_id_in_category(code, AccountCategory::OffBalanceSheet)
                    .ok_or_else(|| {
                        core_accounting::chart_of_accounts::error::ChartOfAccountsError::InvalidAccountCategory {
                            code: code.clone(),
                            category: AccountCategory::OffBalanceSheet,
                        }
                        .into()
                    })
        };

        let revenue_account_set_member_parent_id = |code: &AccountCode| -> Result<
            CalaAccountSetId,
            ChartOfAccountsIntegrationError,
        > {
            chart
                    .find_account_set_id_in_category(code, AccountCategory::Revenue)
                    .ok_or_else(|| {
                        core_accounting::chart_of_accounts::error::ChartOfAccountsError::InvalidAccountCategory {
                            code: code.clone(),
                            category: AccountCategory::Revenue,
                        }
                        .into()
                    })
        };

        let asset_account_set_member_parent_id = |code: &AccountCode| -> Result<
            CalaAccountSetId,
            ChartOfAccountsIntegrationError,
        > {
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

        let facility_omnibus_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                chart_of_account_facility_omnibus_parent_code,
            )?;
        let collateral_omnibus_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                chart_of_account_collateral_omnibus_parent_code,
            )?;
        let liquidation_proceeds_omnibus_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                chart_of_account_liquidation_proceeds_omnibus_parent_code,
            )?;
        let payments_made_omnibus_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                chart_of_account_payments_made_omnibus_parent_code,
            )?;
        let interest_added_to_obligations_omnibus_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                chart_of_account_interest_added_to_obligations_omnibus_parent_code,
            )?;
        let facility_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(chart_of_account_facility_parent_code)?;
        let collateral_parent_account_set_id = off_balance_sheet_account_set_member_parent_id(
            chart_of_account_collateral_parent_code,
        )?;
        let collateral_in_liquidation_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                chart_of_account_collateral_in_liquidation_parent_code,
            )?;

        let interest_income_parent_account_set_id =
            revenue_account_set_member_parent_id(chart_of_account_interest_income_parent_code)?;
        let fee_income_parent_account_set_id =
            revenue_account_set_member_parent_id(chart_of_account_fee_income_parent_code)?;
        let payment_holding_parent_account_set_id =
            asset_account_set_member_parent_id(chart_of_account_payment_holding_parent_code)?;
        let uncovered_outstanding_parent_account_set_id =
            off_balance_sheet_account_set_member_parent_id(
                chart_of_account_uncovered_outstanding_parent_code,
            )?;

        let short_term_disbursed_integration_meta = ShortTermDisbursedIntegrationMeta {
            short_term_individual_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_short_term_individual_disbursed_receivable_parent_code,
                )?,
            short_term_government_entity_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_short_term_government_entity_disbursed_receivable_parent_code,
                )?,
            short_term_private_company_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_short_term_private_company_disbursed_receivable_parent_code,
                )?,
            short_term_bank_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_short_term_bank_disbursed_receivable_parent_code,
                )?,
            short_term_financial_institution_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code,
                )?,
            short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
                )?,
            short_term_non_domiciled_company_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code,
                )?,
        };

        let long_term_disbursed_integration_meta = LongTermDisbursedIntegrationMeta {
            long_term_individual_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_long_term_individual_disbursed_receivable_parent_code,
                )?,
            long_term_government_entity_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_long_term_government_entity_disbursed_receivable_parent_code,
                )?,
            long_term_private_company_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_long_term_private_company_disbursed_receivable_parent_code,
                )?,
            long_term_bank_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_long_term_bank_disbursed_receivable_parent_code,
                )?,
            long_term_financial_institution_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code,
                )?,
            long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
                )?,
            long_term_non_domiciled_company_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code,
                )?,
        };

        let short_term_interest_integration_meta = ShortTermInterestIntegrationMeta {
            short_term_individual_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_short_term_individual_interest_receivable_parent_code,
                )?,
            short_term_government_entity_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_short_term_government_entity_interest_receivable_parent_code,
                )?,
            short_term_private_company_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_short_term_private_company_interest_receivable_parent_code,
                )?,
            short_term_bank_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_short_term_bank_interest_receivable_parent_code,
                )?,
            short_term_financial_institution_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_short_term_financial_institution_interest_receivable_parent_code,
                )?,
            short_term_foreign_agency_or_subsidiary_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
                )?,
            short_term_non_domiciled_company_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code,
                )?,
        };

        let long_term_interest_integration_meta = LongTermInterestIntegrationMeta {
            long_term_individual_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_long_term_individual_interest_receivable_parent_code,
                )?,
            long_term_government_entity_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_long_term_government_entity_interest_receivable_parent_code,
                )?,
            long_term_private_company_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_long_term_private_company_interest_receivable_parent_code,
                )?,
            long_term_bank_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_long_term_bank_interest_receivable_parent_code,
                )?,
            long_term_financial_institution_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_long_term_financial_institution_interest_receivable_parent_code,
                )?,
            long_term_foreign_agency_or_subsidiary_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
                )?,
            long_term_non_domiciled_company_interest_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code,
                )?,
        };

        let overdue_disbursed_integration_meta = OverdueDisbursedIntegrationMeta {
            overdue_individual_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_overdue_individual_disbursed_receivable_parent_code,
                )?,
            overdue_government_entity_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_overdue_government_entity_disbursed_receivable_parent_code,
                )?,
            overdue_private_company_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_overdue_private_company_disbursed_receivable_parent_code,
                )?,
            overdue_bank_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_overdue_bank_disbursed_receivable_parent_code,
                )?,
            overdue_financial_institution_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code,
                )?,
            overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
                )?,
            overdue_non_domiciled_company_disbursed_receivable_parent_account_set_id:
                asset_account_set_member_parent_id(
                    chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code,
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
