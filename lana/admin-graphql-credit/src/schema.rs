use async_graphql::{Context, Object, types::connection::*};

use admin_graphql_shared::accounting::CHART_REF;
use admin_graphql_shared::primitives::*;
use lana_app::credit::LiquidationsByIdCursor;

use crate::{
    collateral::*, contract_creation::*, credit_config::*, credit_facility::*, disbursal::*,
    liquidation::*, pending_facility::*, proposal::*, terms_template::*,
};

#[derive(Default)]
pub struct CreditQuery;

#[Object]
impl CreditQuery {
    async fn credit_facility(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<CreditFacilityBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            CreditFacilityBase,
            app.credit().facilities().find_by_id(sub, id)
        )
    }

    async fn credit_facility_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<CreditFacilityBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            CreditFacilityBase,
            app.credit().facilities().find_by_public_id(sub, id)
        )
    }

    async fn credit_facilities(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
        #[graphql(default_with = "Some(CreditFacilitiesSort::default())")] sort: Option<
            CreditFacilitiesSort,
        >,
        filter: Option<CreditFacilitiesFilter>,
    ) -> async_graphql::Result<
        Connection<CreditFacilitiesCursor, CreditFacilityBase, EmptyFields, EmptyFields>,
    > {
        let filter = DomainCreditFacilitiesFilters {
            status: filter.as_ref().and_then(|f| f.status),
            collateralization_state: filter.as_ref().and_then(|f| f.collateralization_state),
            customer_id: None,
        };
        let sort = sort.unwrap_or_default();
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_combo_cursor!(
            CreditFacilitiesCursor,
            CreditFacilityBase,
            DomainCreditFacilitiesSortBy::from(sort),
            after,
            first,
            |query| app.credit().facilities().list(sub, query, filter, sort)
        )
    }

    async fn credit_facility_proposal(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<CreditFacilityProposalBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            CreditFacilityProposalBase,
            app.credit().proposals().find_by_id(sub, id)
        )
    }

    async fn credit_facility_proposals(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<
            CreditFacilityProposalsByCreatedAtCursor,
            CreditFacilityProposalBase,
            EmptyFields,
            EmptyFields,
        >,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            CreditFacilityProposalsByCreatedAtCursor,
            CreditFacilityProposalBase,
            after,
            first,
            |query| app.credit().proposals().list(sub, query)
        )
    }

    async fn pending_credit_facility(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<PendingCreditFacilityBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            PendingCreditFacilityBase,
            app.credit().pending_credit_facilities().find_by_id(sub, id)
        )
    }

    async fn pending_credit_facilities(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<
            PendingCreditFacilitiesByCreatedAtCursor,
            PendingCreditFacilityBase,
            EmptyFields,
            EmptyFields,
        >,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            PendingCreditFacilitiesByCreatedAtCursor,
            PendingCreditFacilityBase,
            after,
            first,
            |query| app.credit().pending_credit_facilities().list(sub, query)
        )
    }

    async fn disbursal(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<CreditFacilityDisbursalBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            CreditFacilityDisbursalBase,
            app.credit().disbursals().find_by_id(sub, id)
        )
    }

    async fn disbursal_by_public_id(
        &self,
        ctx: &Context<'_>,
        id: PublicId,
    ) -> async_graphql::Result<Option<CreditFacilityDisbursalBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            CreditFacilityDisbursalBase,
            app.credit().disbursals().find_by_public_id(sub, id)
        )
    }

    async fn disbursals(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<DisbursalsCursor, CreditFacilityDisbursalBase, EmptyFields, EmptyFields>,
    > {
        let filter = DisbursalsFilters::default();
        let sort = Sort {
            by: DomainDisbursalsSortBy::CreatedAt,
            direction: ListDirection::Descending,
        };
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_combo_cursor!(
            DisbursalsCursor,
            CreditFacilityDisbursalBase,
            sort.by,
            after,
            first,
            |query| { app.credit().disbursals().list(sub, query, filter, sort) }
        )
    }

    async fn liquidation(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<LiquidationBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(
            LiquidationBase,
            app.credit().collaterals().find_liquidation_by_id(sub, id)
        )
    }

    async fn liquidations(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<LiquidationsByIdCursor, LiquidationBase, EmptyFields, EmptyFields>,
    > {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        list_with_cursor!(
            LiquidationsByIdCursor,
            LiquidationBase,
            after,
            first,
            |query| app.credit().collaterals().list_liquidations(sub, query)
        )
    }

    async fn terms_template(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<TermsTemplate>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        maybe_fetch_one!(TermsTemplate, app.terms_templates().find_by_id(sub, id))
    }

    async fn terms_templates(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<TermsTemplate>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let terms_templates = app.terms_templates().list(sub).await?;
        Ok(terms_templates
            .into_iter()
            .map(TermsTemplate::from)
            .collect())
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

    async fn loan_agreement(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<LoanAgreement>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let agreement = app.contract_creation().find_by_id(sub, id).await?;
        Ok(agreement.map(LoanAgreement::from))
    }
}

mutation_payload! { CreditFacilityPartialPaymentRecordPayload, credit_facility: CreditFacilityBase }
mutation_payload! { CreditFacilityCompletePayload, credit_facility: CreditFacilityBase }
mutation_payload! { CreditFacilityProposalCreatePayload, credit_facility_proposal: CreditFacilityProposalBase }
mutation_payload! { CreditFacilityProposalCustomerApprovalConcludePayload, credit_facility_proposal: CreditFacilityProposalBase }
mutation_payload! { CreditFacilityDisbursalInitiatePayload, disbursal: CreditFacilityDisbursalBase }
mutation_payload! { CollateralUpdatePayload, collateral: CollateralBase }
mutation_payload! { CollateralRecordSentToLiquidationPayload, collateral: CollateralBase }
mutation_payload! { CollateralRecordProceedsFromLiquidationPayload, collateral: CollateralBase }

#[derive(Default)]
pub struct CreditMutation;

#[Object]
impl CreditMutation {
    pub async fn credit_facility_proposal_create(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityProposalCreateInput,
    ) -> async_graphql::Result<CreditFacilityProposalCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let CreditFacilityProposalCreateInput {
            facility,
            customer_id,
            terms,
            custodian_id,
        } = input;

        let credit_facility_term_values = lana_app::terms::TermValues::builder()
            .annual_rate(terms.annual_rate)
            .accrual_interval(terms.accrual_interval)
            .accrual_cycle_interval(terms.accrual_cycle_interval)
            .one_time_fee_rate(terms.one_time_fee_rate)
            .disbursal_policy(terms.disbursal_policy)
            .duration(terms.duration)
            .interest_due_duration_from_accrual(terms.interest_due_duration_from_accrual)
            .obligation_overdue_duration_from_due(terms.obligation_overdue_duration_from_due)
            .obligation_liquidation_duration_from_due(
                terms.obligation_liquidation_duration_from_due,
            )
            .liquidation_cvl(terms.liquidation_cvl)
            .margin_call_cvl(terms.margin_call_cvl)
            .initial_cvl(terms.initial_cvl)
            .build()?;

        exec_mutation!(
            CreditFacilityProposalCreatePayload,
            CreditFacilityProposalBase,
            app.create_facility_proposal(
                sub,
                customer_id,
                facility,
                credit_facility_term_values,
                custodian_id
            )
        )
    }

    pub async fn credit_facility_proposal_customer_approval_conclude(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityProposalCustomerApprovalConcludeInput,
    ) -> async_graphql::Result<CreditFacilityProposalCustomerApprovalConcludePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let CreditFacilityProposalCustomerApprovalConcludeInput {
            credit_facility_proposal_id,
            approved,
        } = input;
        exec_mutation!(
            CreditFacilityProposalCustomerApprovalConcludePayload,
            CreditFacilityProposalBase,
            app.credit().proposals().conclude_customer_approval(
                sub,
                credit_facility_proposal_id,
                approved
            )
        )
    }

    pub async fn collateral_update(
        &self,
        ctx: &Context<'_>,
        input: CollateralUpdateInput,
    ) -> async_graphql::Result<CollateralUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let CollateralUpdateInput {
            collateral_id,
            collateral,
            effective,
        } = input;
        exec_mutation!(
            CollateralUpdatePayload,
            CollateralBase,
            app.credit().collaterals().update_collateral_by_id(
                sub,
                collateral_id.into(),
                collateral,
                effective.into()
            )
        )
    }

    pub async fn credit_facility_partial_payment_record(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityPartialPaymentRecordInput,
    ) -> async_graphql::Result<CreditFacilityPartialPaymentRecordPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CreditFacilityPartialPaymentRecordPayload,
            CreditFacilityBase,
            app.record_payment(sub, input.credit_facility_id, input.amount)
        )
    }

    pub async fn credit_facility_partial_payment_with_date_record(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityPartialPaymentWithDateRecordInput,
    ) -> async_graphql::Result<CreditFacilityPartialPaymentRecordPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CreditFacilityPartialPaymentRecordPayload,
            CreditFacilityBase,
            app.record_payment_with_date(
                sub,
                input.credit_facility_id,
                input.amount,
                input.effective
            )
        )
    }

    pub async fn credit_facility_disbursal_initiate(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityDisbursalInitiateInput,
    ) -> async_graphql::Result<CreditFacilityDisbursalInitiatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CreditFacilityDisbursalInitiatePayload,
            CreditFacilityDisbursalBase,
            app.credit()
                .initiate_disbursal(sub, input.credit_facility_id.into(), input.amount)
        )
    }

    async fn credit_facility_complete(
        &self,
        ctx: &Context<'_>,
        input: CreditFacilityCompleteInput,
    ) -> async_graphql::Result<CreditFacilityCompletePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CreditFacilityCompletePayload,
            CreditFacilityBase,
            app.credit()
                .complete_facility(sub, input.credit_facility_id)
        )
    }

    async fn collateral_record_sent_to_liquidation(
        &self,
        ctx: &Context<'_>,
        input: CollateralRecordSentToLiquidationInput,
    ) -> async_graphql::Result<CollateralRecordSentToLiquidationPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CollateralRecordSentToLiquidationPayload,
            CollateralBase,
            app.credit()
                .collaterals()
                .record_collateral_update_via_liquidation(
                    sub,
                    input.collateral_id.into(),
                    input.amount
                )
        )
    }

    async fn collateral_record_proceeds_from_liquidation(
        &self,
        ctx: &Context<'_>,
        input: CollateralRecordProceedsFromLiquidationInput,
    ) -> async_graphql::Result<CollateralRecordProceedsFromLiquidationPayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        exec_mutation!(
            CollateralRecordProceedsFromLiquidationPayload,
            CollateralBase,
            app.credit()
                .collaterals()
                .record_proceeds_received_and_liquidation_completed(
                    sub,
                    input.collateral_id.into(),
                    input.amount
                )
        )
    }

    async fn terms_template_create(
        &self,
        ctx: &Context<'_>,
        input: TermsTemplateCreateInput,
    ) -> async_graphql::Result<TermsTemplateCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let term_values = lana_app::terms::TermValues::builder()
            .annual_rate(input.annual_rate)
            .accrual_interval(input.accrual_interval)
            .accrual_cycle_interval(input.accrual_cycle_interval)
            .one_time_fee_rate(input.one_time_fee_rate)
            .disbursal_policy(input.disbursal_policy)
            .duration(input.duration)
            .interest_due_duration_from_accrual(input.interest_due_duration_from_accrual)
            .obligation_overdue_duration_from_due(input.obligation_overdue_duration_from_due)
            .obligation_liquidation_duration_from_due(
                input.obligation_liquidation_duration_from_due,
            )
            .liquidation_cvl(input.liquidation_cvl)
            .margin_call_cvl(input.margin_call_cvl)
            .initial_cvl(input.initial_cvl)
            .build()?;

        exec_mutation!(
            TermsTemplateCreatePayload,
            TermsTemplate,
            app.terms_templates()
                .create_terms_template(sub, input.name, term_values)
        )
    }

    async fn terms_template_update(
        &self,
        ctx: &Context<'_>,
        input: TermsTemplateUpdateInput,
    ) -> async_graphql::Result<TermsTemplateUpdatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let term_values = lana_app::terms::TermValues::builder()
            .annual_rate(input.annual_rate)
            .accrual_interval(input.accrual_interval)
            .accrual_cycle_interval(input.accrual_cycle_interval)
            .one_time_fee_rate(input.one_time_fee_rate)
            .disbursal_policy(input.disbursal_policy)
            .duration(input.duration)
            .interest_due_duration_from_accrual(input.interest_due_duration_from_accrual)
            .obligation_overdue_duration_from_due(input.obligation_overdue_duration_from_due)
            .obligation_liquidation_duration_from_due(
                input.obligation_liquidation_duration_from_due,
            )
            .liquidation_cvl(input.liquidation_cvl)
            .margin_call_cvl(input.margin_call_cvl)
            .initial_cvl(input.initial_cvl)
            .build()?;
        exec_mutation!(
            TermsTemplateUpdatePayload,
            TermsTemplate,
            app.terms_templates().update_term_values(
                sub,
                TermsTemplateId::from(input.id),
                term_values
            )
        )
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
            .find_by_reference(CHART_REF)
            .await?;

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
            chart_of_account_uncovered_outstanding_parent_code: chart_of_account_uncovered_outstanding_parent_code
                .parse()?,
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
                chart_of_account_long_term_bank_disbursed_receivable_parent_code
                    .parse()?,
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
                chart_of_account_short_term_individual_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_government_entity_interest_receivable_parent_code:
                chart_of_account_short_term_government_entity_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_private_company_interest_receivable_parent_code:
                chart_of_account_short_term_private_company_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_short_term_bank_interest_receivable_parent_code:
                chart_of_account_short_term_bank_interest_receivable_parent_code
                    .parse()?,
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
                chart_of_account_long_term_individual_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_government_entity_interest_receivable_parent_code:
                chart_of_account_long_term_government_entity_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_private_company_interest_receivable_parent_code:
                chart_of_account_long_term_private_company_interest_receivable_parent_code
                    .parse()?,
            chart_of_account_long_term_bank_interest_receivable_parent_code:
                chart_of_account_long_term_bank_interest_receivable_parent_code
                    .parse()?,
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
                chart_of_account_overdue_individual_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_government_entity_disbursed_receivable_parent_code:
                chart_of_account_overdue_government_entity_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_private_company_disbursed_receivable_parent_code:
                chart_of_account_overdue_private_company_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_bank_disbursed_receivable_parent_code:
                chart_of_account_overdue_bank_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code:
                chart_of_account_overdue_financial_institution_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code:
                chart_of_account_overdue_foreign_agency_or_subsidiary_disbursed_receivable_parent_code
                    .parse()?,
            chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code:
                chart_of_account_overdue_non_domiciled_company_disbursed_receivable_parent_code
                    .parse()?
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

    pub async fn loan_agreement_generate(
        &self,
        ctx: &Context<'_>,
        input: LoanAgreementGenerateInput,
    ) -> async_graphql::Result<LoanAgreementGeneratePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        let loan_agreement = app
            .contract_creation()
            .initiate_loan_agreement_generation(sub, input.customer_id)
            .await?;

        let loan_agreement = LoanAgreement::from(loan_agreement);
        Ok(LoanAgreementGeneratePayload::from(loan_agreement))
    }

    async fn loan_agreement_download_link_generate(
        &self,
        ctx: &Context<'_>,
        input: LoanAgreementDownloadLinksGenerateInput,
    ) -> async_graphql::Result<LoanAgreementDownloadLinksGeneratePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let doc = app
            .contract_creation()
            .generate_document_download_link(sub, input.loan_agreement_id)
            .await?;
        Ok(LoanAgreementDownloadLinksGeneratePayload::from(doc))
    }
}
