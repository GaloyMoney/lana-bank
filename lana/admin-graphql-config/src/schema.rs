use async_graphql::{Context, Object, types::connection::*};

use super::{credit_config::*, deposit_config::*, domain_config::*};

const CHART_REF: &str = lana_app::accounting_init::constants::CHART_REF;

#[derive(Default)]
pub struct ConfigQuery;

#[Object]
impl ConfigQuery {
    async fn deposit_config(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<DepositModuleConfig>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let config = app
            .deposits()
            .chart_of_accounts_integrations()
            .get_config(sub)
            .await?;
        Ok(config.map(DepositModuleConfig::from))
    }

    async fn domain_configs(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<DomainConfigsByKeyCursor, DomainConfig, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            DomainConfigsByKeyCursor,
            DomainConfig,
            after,
            first,
            |query| app.exposed_domain_configs().list(sub, query)
        )
    }

    async fn credit_config(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<CreditModuleConfig>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let config = app
            .credit()
            .chart_of_accounts_integrations()
            .get_config(sub)
            .await?;
        Ok(config.map(CreditModuleConfig::from))
    }
}

#[derive(Default)]
pub struct ConfigMutation;

#[Object]
impl ConfigMutation {
    async fn domain_config_update(
        &self,
        ctx: &Context<'_>,
        input: DomainConfigUpdateInput,
    ) -> async_graphql::Result<DomainConfigUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            DomainConfigUpdatePayload,
            DomainConfig,
            app.exposed_domain_configs().update_from_json(
                sub,
                input.domain_config_id,
                input.value.into_inner(),
            )
        )
    }

    async fn deposit_module_configure(
        &self,
        ctx: &Context<'_>,
        input: DepositModuleConfigureInput,
    ) -> async_graphql::Result<DepositModuleConfigurePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let chart = app
            .accounting()
            .chart_of_accounts()
            .maybe_find_by_reference(CHART_REF)
            .await?
            .unwrap_or_else(|| panic!("Chart of accounts not found for ref {CHART_REF:?}"));

        let DepositModuleConfigureInput {
            chart_of_accounts_omnibus_parent_code,
            chart_of_accounts_individual_deposit_accounts_parent_code,
            chart_of_accounts_government_entity_deposit_accounts_parent_code,
            chart_of_account_private_company_deposit_accounts_parent_code,
            chart_of_account_bank_deposit_accounts_parent_code,
            chart_of_account_financial_institution_deposit_accounts_parent_code,
            chart_of_account_non_domiciled_company_deposit_accounts_parent_code,
            chart_of_accounts_frozen_individual_deposit_accounts_parent_code,
            chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code,
            chart_of_account_frozen_private_company_deposit_accounts_parent_code,
            chart_of_account_frozen_bank_deposit_accounts_parent_code,
            chart_of_account_frozen_financial_institution_deposit_accounts_parent_code,
            chart_of_account_frozen_non_domiciled_company_deposit_accounts_parent_code,
        } = input;

        let config_values = lana_app::deposit::ChartOfAccountsIntegrationConfig {
            chart_of_accounts_id: chart.id,
            chart_of_accounts_individual_deposit_accounts_parent_code:
                chart_of_accounts_individual_deposit_accounts_parent_code.parse()?,
            chart_of_accounts_government_entity_deposit_accounts_parent_code:
                chart_of_accounts_government_entity_deposit_accounts_parent_code.parse()?,
            chart_of_account_private_company_deposit_accounts_parent_code:
                chart_of_account_private_company_deposit_accounts_parent_code.parse()?,
            chart_of_account_bank_deposit_accounts_parent_code:
                chart_of_account_bank_deposit_accounts_parent_code.parse()?,
            chart_of_account_financial_institution_deposit_accounts_parent_code:
                chart_of_account_financial_institution_deposit_accounts_parent_code.parse()?,
            chart_of_account_non_domiciled_company_deposit_accounts_parent_code:
                chart_of_account_non_domiciled_company_deposit_accounts_parent_code.parse()?,
            chart_of_accounts_frozen_individual_deposit_accounts_parent_code:
                chart_of_accounts_frozen_individual_deposit_accounts_parent_code.parse()?,
            chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code:
                chart_of_accounts_frozen_government_entity_deposit_accounts_parent_code.parse()?,
            chart_of_account_frozen_private_company_deposit_accounts_parent_code:
                chart_of_account_frozen_private_company_deposit_accounts_parent_code.parse()?,
            chart_of_account_frozen_bank_deposit_accounts_parent_code:
                chart_of_account_frozen_bank_deposit_accounts_parent_code.parse()?,
            chart_of_account_frozen_financial_institution_deposit_accounts_parent_code:
                chart_of_account_frozen_financial_institution_deposit_accounts_parent_code
                    .parse()?,
            chart_of_account_frozen_non_domiciled_company_deposit_accounts_parent_code:
                chart_of_account_frozen_non_domiciled_company_deposit_accounts_parent_code
                    .parse()?,
            chart_of_accounts_omnibus_parent_code: chart_of_accounts_omnibus_parent_code.parse()?,
        };

        let config = app
            .deposits()
            .chart_of_accounts_integrations()
            .set_config(sub, &chart, config_values)
            .await?;
        Ok(DepositModuleConfigurePayload::from(
            DepositModuleConfig::from(config),
        ))
    }

    async fn credit_module_configure(
        &self,
        ctx: &Context<'_>,
        input: CreditModuleConfigureInput,
    ) -> async_graphql::Result<CreditModuleConfigurePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let chart = app
            .accounting()
            .chart_of_accounts()
            .maybe_find_by_reference(CHART_REF)
            .await?
            .unwrap_or_else(|| panic!("Chart of accounts not found for ref {CHART_REF:?}"));

        let CreditModuleConfigureInput {
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
        } = input;

        let config_values = lana_app::credit::ChartOfAccountsIntegrationConfig {
            chart_of_accounts_id: chart.id,
            chart_of_account_facility_omnibus_parent_code:
                chart_of_account_facility_omnibus_parent_code.parse()?,
            chart_of_account_collateral_omnibus_parent_code:
                chart_of_account_collateral_omnibus_parent_code.parse()?,
            chart_of_account_payments_made_omnibus_parent_code:
                chart_of_account_payments_made_omnibus_parent_code.parse()?,
            chart_of_account_interest_added_to_obligations_omnibus_parent_code:
                chart_of_account_interest_added_to_obligations_omnibus_parent_code.parse()?,
            chart_of_account_liquidation_proceeds_omnibus_parent_code:
                chart_of_account_liquidation_proceeds_omnibus_parent_code.parse()?,
            chart_of_account_facility_parent_code: chart_of_account_facility_parent_code.parse()?,
            chart_of_account_collateral_parent_code: chart_of_account_collateral_parent_code
                .parse()?,
            chart_of_account_collateral_in_liquidation_parent_code:
                chart_of_account_collateral_in_liquidation_parent_code.parse()?,
            chart_of_account_liquidated_collateral_parent_code:
                chart_of_account_liquidated_collateral_parent_code.parse()?,
            chart_of_account_proceeds_from_liquidation_parent_code:
                chart_of_account_proceeds_from_liquidation_parent_code.parse()?,
            chart_of_account_interest_income_parent_code:
                chart_of_account_interest_income_parent_code.parse()?,
            chart_of_account_fee_income_parent_code: chart_of_account_fee_income_parent_code
                .parse()?,
            chart_of_account_payment_holding_parent_code: chart_of_account_payment_holding_parent_code
                .parse()?,
            chart_of_account_uncovered_outstanding_parent_code:
                chart_of_account_uncovered_outstanding_parent_code.parse()?,
            chart_of_account_disbursed_defaulted_parent_code:
                chart_of_account_disbursed_defaulted_parent_code.parse()?,
            chart_of_account_interest_defaulted_parent_code:
                chart_of_account_interest_defaulted_parent_code.parse()?,
            chart_of_account_short_term_individual_disbursed_receivable_parent_code:
                chart_of_account_short_term_individual_disbursed_receivable_parent_code.parse()?,
            chart_of_account_short_term_government_entity_disbursed_receivable_parent_code:
                chart_of_account_short_term_government_entity_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_private_company_disbursed_receivable_parent_code:
                chart_of_account_short_term_private_company_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_bank_disbursed_receivable_parent_code:
                chart_of_account_short_term_bank_disbursed_receivable_parent_code.parse()?,
            chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code:
                chart_of_account_short_term_financial_institution_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                chart_of_account_short_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code:
                chart_of_account_short_term_non_domiciled_company_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_individual_disbursed_receivable_parent_code:
                chart_of_account_long_term_individual_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_government_entity_disbursed_receivable_parent_code:
                chart_of_account_long_term_government_entity_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_private_company_disbursed_receivable_parent_code:
                chart_of_account_long_term_private_company_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_bank_disbursed_receivable_parent_code:
                chart_of_account_long_term_bank_disbursed_receivable_parent_code.parse()?,
            chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code:
                chart_of_account_long_term_financial_institution_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                chart_of_account_long_term_foreign_agency_or_subsidiary_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code:
                chart_of_account_long_term_non_domiciled_company_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_individual_interest_receivable_parent_code:
                chart_of_account_short_term_individual_interest_receivable_parent_code.parse()?,
            chart_of_account_short_term_government_entity_interest_receivable_parent_code:
                chart_of_account_short_term_government_entity_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_private_company_interest_receivable_parent_code:
                chart_of_account_short_term_private_company_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_bank_interest_receivable_parent_code:
                chart_of_account_short_term_bank_interest_receivable_parent_code.parse()?,
            chart_of_account_short_term_financial_institution_interest_receivable_parent_code:
                chart_of_account_short_term_financial_institution_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
                chart_of_account_short_term_foreign_agency_or_subsidiary_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code:
                chart_of_account_short_term_non_domiciled_company_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_individual_interest_receivable_parent_code:
                chart_of_account_long_term_individual_interest_receivable_parent_code.parse()?,
            chart_of_account_long_term_government_entity_interest_receivable_parent_code:
                chart_of_account_long_term_government_entity_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_private_company_interest_receivable_parent_code:
                chart_of_account_long_term_private_company_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_bank_interest_receivable_parent_code:
                chart_of_account_long_term_bank_interest_receivable_parent_code.parse()?,
            chart_of_account_long_term_financial_institution_interest_receivable_parent_code:
                chart_of_account_long_term_financial_institution_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code:
                chart_of_account_long_term_foreign_agency_or_subsidiary_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code:
                chart_of_account_long_term_non_domiciled_company_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_individual_disbursed_receivable_parent_code:
                chart_of_account_overdue_individual_disbursed_receivable_parent_code.parse()?,
            chart_of_account_overdue_government_entity_disbursed_receivable_parent_code:
                chart_of_account_overdue_government_entity_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_private_company_disbursed_receivable_parent_code:
                chart_of_account_overdue_private_company_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_bank_disbursed_receivable_parent_code:
                chart_of_account_overdue_bank_disbursed_receivable_parent_code.parse()?,
            chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code:
                chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code:
                chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code
                    .parse()?,
        };

        let config = app
            .credit()
            .chart_of_accounts_integrations()
            .set_config(sub, &chart, config_values)
            .await?;
        Ok(CreditModuleConfigurePayload::from(
            CreditModuleConfig::from(config),
        ))
    }
}
