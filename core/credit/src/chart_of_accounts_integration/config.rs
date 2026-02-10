use serde::{Deserialize, Serialize};

use core_accounting::{AccountCode, CalaAccountSetId, Chart, ChartId};
use domain_config::define_internal_config;

use super::error::ChartOfAccountsIntegrationError;
use crate::{
    ledger::{
        CreditLedger, LongTermDisbursedIntegrationMeta, LongTermInterestIntegrationMeta,
        OverdueDisbursedIntegrationMeta, ShortTermDisbursedIntegrationMeta,
        ShortTermInterestIntegrationMeta,
    },
    primitives::account_sets::CreditAccountCategory,
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
    pub chart_of_account_liquidated_collateral_parent_code: AccountCode,
    pub chart_of_account_proceeds_from_liquidation_parent_code: AccountCode,
    pub chart_of_account_interest_income_parent_code: AccountCode,
    pub chart_of_account_fee_income_parent_code: AccountCode,
    pub chart_of_account_payment_holding_parent_code: AccountCode,
    pub chart_of_account_uncovered_outstanding_parent_code: AccountCode,
    pub chart_of_account_disbursed_defaulted_parent_code: AccountCode,
    pub chart_of_account_interest_defaulted_parent_code: AccountCode,

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
        pub(crate) liquidated_collateral_parent_account_set_id: CalaAccountSetId,
        pub(crate) proceeds_from_liquidation_parent_account_set_id: CalaAccountSetId,
        pub(crate) interest_income_parent_account_set_id: CalaAccountSetId,
        pub(crate) fee_income_parent_account_set_id: CalaAccountSetId,
        pub(crate) payment_holding_parent_account_set_id: CalaAccountSetId,
        pub(crate) uncovered_outstanding_parent_account_set_id: CalaAccountSetId,
        pub(crate) disbursed_defaulted_parent_account_set_id: CalaAccountSetId,
        pub(crate) interest_defaulted_parent_account_set_id: CalaAccountSetId,

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
        ledger: &CreditLedger,
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
            chart_of_account_liquidated_collateral_parent_code,
            chart_of_account_proceeds_from_liquidation_parent_code,
            chart_of_account_interest_income_parent_code,
            chart_of_account_fee_income_parent_code,
            chart_of_account_payment_holding_parent_code,
            chart_of_account_uncovered_outstanding_parent_code,
            chart_of_account_disbursed_defaulted_parent_code,
            chart_of_account_interest_defaulted_parent_code,
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
        let category_account_set_member_parent_id = |code: &AccountCode,
                                                     category: CreditAccountCategory|
         -> Result<
            CalaAccountSetId,
            ChartOfAccountsIntegrationError,
        > {
            chart
                    .find_account_set_id_in_category(code, category.into())
                    .ok_or_else(|| {
                        core_accounting::chart_of_accounts::error::ChartOfAccountsError::InvalidAccountCategory {
                            code: code.clone(),
                            category: category.into(),
                        }
                        .into()
                    })
        };

        let internal_account_sets = ledger.internal_account_sets();

        let facility_omnibus_parent_account_set_id = category_account_set_member_parent_id(
            chart_of_account_facility_omnibus_parent_code,
            ledger.facility_omnibus_account_ids().account_category,
        )?;
        let payments_made_omnibus_parent_account_set_id = category_account_set_member_parent_id(
            chart_of_account_payments_made_omnibus_parent_code,
            ledger.payments_made_omnibus_account_ids().account_category,
        )?;
        let interest_added_to_obligations_omnibus_parent_account_set_id =
            category_account_set_member_parent_id(
                chart_of_account_interest_added_to_obligations_omnibus_parent_code,
                ledger
                    .interest_added_to_obligations_omnibus_account_ids()
                    .account_category,
            )?;
        let collateral_omnibus_parent_account_set_id = category_account_set_member_parent_id(
            chart_of_account_collateral_omnibus_parent_code,
            ledger.collateral_omnibus_account_ids().account_category,
        )?;
        let liquidation_proceeds_omnibus_parent_account_set_id =
            category_account_set_member_parent_id(
                chart_of_account_liquidation_proceeds_omnibus_parent_code,
                ledger
                    .liquidation_proceeds_omnibus_account_ids()
                    .account_category,
            )?;
        let facility_parent_account_set_id = category_account_set_member_parent_id(
            chart_of_account_facility_parent_code,
            internal_account_sets.facility.account_category(),
        )?;
        let collateral_parent_account_set_id = category_account_set_member_parent_id(
            chart_of_account_collateral_parent_code,
            internal_account_sets.collateral.account_category(),
        )?;
        let collateral_in_liquidation_parent_account_set_id =
            category_account_set_member_parent_id(
                chart_of_account_collateral_in_liquidation_parent_code,
                internal_account_sets
                    .liquidation
                    .collateral_in_liquidation
                    .account_category(),
            )?;
        let liquidated_collateral_parent_account_set_id = category_account_set_member_parent_id(
            chart_of_account_liquidated_collateral_parent_code,
            internal_account_sets
                .liquidation
                .liquidated_collateral
                .account_category(),
        )?;
        let proceeds_from_liquidation_parent_account_set_id =
            category_account_set_member_parent_id(
                chart_of_account_proceeds_from_liquidation_parent_code,
                internal_account_sets
                    .liquidation
                    .proceeds_from_liquidation
                    .account_category(),
            )?;

        let interest_income_parent_account_set_id = category_account_set_member_parent_id(
            chart_of_account_interest_income_parent_code,
            internal_account_sets.interest_income.account_category(),
        )?;
        let fee_income_parent_account_set_id = category_account_set_member_parent_id(
            chart_of_account_fee_income_parent_code,
            internal_account_sets.fee_income.account_category(),
        )?;
        let payment_holding_parent_account_set_id = category_account_set_member_parent_id(
            chart_of_account_payment_holding_parent_code,
            internal_account_sets.payment_holding.account_category(),
        )?;
        let uncovered_outstanding_parent_account_set_id = category_account_set_member_parent_id(
            chart_of_account_uncovered_outstanding_parent_code,
            internal_account_sets
                .uncovered_outstanding
                .account_category(),
        )?;

        let disbursed_defaulted_parent_account_set_id = category_account_set_member_parent_id(
            chart_of_account_disbursed_defaulted_parent_code,
            internal_account_sets.disbursed_defaulted.account_category(),
        )?;
        let interest_defaulted_parent_account_set_id = category_account_set_member_parent_id(
            chart_of_account_interest_defaulted_parent_code,
            internal_account_sets.interest_defaulted.account_category(),
        )?;

        let short_term_disbursed_integration_meta = ShortTermDisbursedIntegrationMeta {
            short_term_individual_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_short_term_individual_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .short_term()
                        .individual()
                        .account_category(),
                )?,
            short_term_government_entity_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_short_term_government_entity_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .short_term()
                        .government_entity()
                        .account_category(),
                )?,
            short_term_private_company_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_short_term_private_company_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .short_term()
                        .private_company()
                        .account_category(),
                )?,
            short_term_bank_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_short_term_bank_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .short_term()
                        .bank()
                        .account_category(),
                )?,
            short_term_financial_institution_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .short_term()
                        .financial_institution()
                        .account_category(),
                )?,
            short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .short_term()
                        .foreign_agency_or_subsidiary()
                        .account_category(),
                )?,
            short_term_non_domiciled_company_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .short_term()
                        .non_domiciled_company()
                        .account_category(),
                )?,
        };

        let long_term_disbursed_integration_meta = LongTermDisbursedIntegrationMeta {
            long_term_individual_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_long_term_individual_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .long_term()
                        .individual()
                        .account_category(),
                )?,
            long_term_government_entity_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_long_term_government_entity_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .long_term()
                        .government_entity()
                        .account_category(),
                )?,
            long_term_private_company_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_long_term_private_company_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .long_term()
                        .private_company()
                        .account_category(),
                )?,
            long_term_bank_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_long_term_bank_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .long_term()
                        .bank()
                        .account_category(),
                )?,
            long_term_financial_institution_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .long_term()
                        .financial_institution()
                        .account_category(),
                )?,
            long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .long_term()
                        .foreign_agency_or_subsidiary()
                        .account_category(),
                )?,
            long_term_non_domiciled_company_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .long_term()
                        .non_domiciled_company()
                        .account_category(),
                )?,
        };

        let short_term_interest_integration_meta = ShortTermInterestIntegrationMeta {
            short_term_individual_interest_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_short_term_individual_interest_receivable_parent_code,
                    internal_account_sets
                        .interest_receivable
                        .short_term()
                        .individual()
                        .account_category(),
                )?,
            short_term_government_entity_interest_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_short_term_government_entity_interest_receivable_parent_code,
                    internal_account_sets
                        .interest_receivable
                        .short_term()
                        .government_entity()
                        .account_category(),
                )?,
            short_term_private_company_interest_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_short_term_private_company_interest_receivable_parent_code,
                    internal_account_sets
                        .interest_receivable
                        .short_term()
                        .private_company()
                        .account_category(),
                )?,
            short_term_bank_interest_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_short_term_bank_interest_receivable_parent_code,
                    internal_account_sets
                        .interest_receivable
                        .short_term()
                        .bank()
                        .account_category(),
                )?,
            short_term_financial_institution_interest_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_short_term_financial_institution_interest_receivable_parent_code,
                    internal_account_sets
                        .interest_receivable
                        .short_term()
                        .financial_institution()
                        .account_category(),
                )?,
            short_term_foreign_agency_or_subsidiary_interest_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
                    internal_account_sets
                        .interest_receivable
                        .short_term()
                        .foreign_agency_or_subsidiary()
                        .account_category(),
                )?,
            short_term_non_domiciled_company_interest_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code,
                    internal_account_sets
                        .interest_receivable
                        .short_term()
                        .non_domiciled_company()
                        .account_category(),
                )?,
        };

        let long_term_interest_integration_meta = LongTermInterestIntegrationMeta {
            long_term_individual_interest_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_long_term_individual_interest_receivable_parent_code,
                    internal_account_sets
                        .interest_receivable
                        .long_term()
                        .individual()
                        .account_category(),
                )?,
            long_term_government_entity_interest_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_long_term_government_entity_interest_receivable_parent_code,
                    internal_account_sets
                        .interest_receivable
                        .long_term()
                        .government_entity()
                        .account_category(),
                )?,
            long_term_private_company_interest_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_long_term_private_company_interest_receivable_parent_code,
                    internal_account_sets
                        .interest_receivable
                        .long_term()
                        .private_company()
                        .account_category(),
                )?,
            long_term_bank_interest_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_long_term_bank_interest_receivable_parent_code,
                    internal_account_sets
                        .interest_receivable
                        .long_term()
                        .bank()
                        .account_category(),
                )?,
            long_term_financial_institution_interest_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_long_term_financial_institution_interest_receivable_parent_code,
                    internal_account_sets
                        .interest_receivable
                        .long_term()
                        .financial_institution()
                        .account_category(),
                )?,
            long_term_foreign_agency_or_subsidiary_interest_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code,
                    internal_account_sets
                        .interest_receivable
                        .long_term()
                        .foreign_agency_or_subsidiary()
                        .account_category(),
                )?,
            long_term_non_domiciled_company_interest_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code,
                    internal_account_sets
                        .interest_receivable
                        .long_term()
                        .non_domiciled_company()
                        .account_category(),
                )?,
        };

        let overdue_disbursed_integration_meta = OverdueDisbursedIntegrationMeta {
            overdue_individual_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_overdue_individual_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .overdue()
                        .individual()
                        .account_category(),
                )?,
            overdue_government_entity_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_overdue_government_entity_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .overdue()
                        .government_entity()
                        .account_category(),
                )?,
            overdue_private_company_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_overdue_private_company_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .overdue()
                        .private_company()
                        .account_category(),
                )?,
            overdue_bank_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_overdue_bank_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .overdue()
                        .bank()
                        .account_category(),
                )?,
            overdue_financial_institution_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .overdue()
                        .financial_institution()
                        .account_category(),
                )?,
            overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .overdue()
                        .foreign_agency_or_subsidiary()
                        .account_category(),
                )?,
            overdue_non_domiciled_company_disbursed_receivable_parent_account_set_id:
                category_account_set_member_parent_id(
                    chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code,
                    internal_account_sets
                        .disbursed_receivable
                        .overdue()
                        .non_domiciled_company()
                        .account_category(),
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
            liquidated_collateral_parent_account_set_id,
            proceeds_from_liquidation_parent_account_set_id,
            interest_income_parent_account_set_id,
            fee_income_parent_account_set_id,
            payment_holding_parent_account_set_id,
            uncovered_outstanding_parent_account_set_id,
            disbursed_defaulted_parent_account_set_id,
            interest_defaulted_parent_account_set_id,

            short_term_disbursed_integration_meta,
            long_term_disbursed_integration_meta,
            short_term_interest_integration_meta,
            long_term_interest_integration_meta,
            overdue_disbursed_integration_meta,
        })
    }
}
