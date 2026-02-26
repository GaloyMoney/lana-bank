use async_graphql::{Context, Object};

use admin_graphql_shared::primitives::*;

use super::{contract_creation::*, terms_template::*};

#[derive(Default)]
pub struct ContractsQuery;

#[Object]
impl ContractsQuery {
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

#[derive(Default)]
pub struct ContractsMutation;

#[Object]
impl ContractsMutation {
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
