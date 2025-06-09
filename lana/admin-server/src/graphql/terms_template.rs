use async_graphql::*;

use crate::primitives::*;

use super::terms::*;

use lana_app::credit::TermsTemplate as DomainTermsTemplate;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct TermsTemplate {
    id: ID,
    terms_id: UUID,
    values: TermValues,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainTermsTemplate>,
}

impl From<DomainTermsTemplate> for TermsTemplate {
    fn from(terms: DomainTermsTemplate) -> Self {
        Self {
            id: terms.id.to_global_id(),
            created_at: terms.created_at().into(),
            terms_id: terms.id.into(),
            values: terms.values.into(),
            entity: Arc::new(terms),
        }
    }
}

#[ComplexObject]
impl TermsTemplate {
    async fn name(&self) -> &str {
        &self.entity.name
    }

    async fn subject_can_update_terms_template(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .terms_templates()
            .subject_can_update_terms_template(sub, false)
            .await
            .is_ok())
    }
}

#[derive(InputObject)]
pub(super) struct TermsTemplateCreateInput {
    pub name: String,
    pub annual_rate: AnnualRatePct,
    pub accrual_interval: InterestInterval,
    pub accrual_cycle_interval: InterestInterval,
    pub one_time_fee_rate: OneTimeFeeRatePct,
    pub duration: DurationInput,
    pub interest_due_duration_from_accrual: DurationInput,
    pub obligation_overdue_duration_from_due: DurationInput,
    pub obligation_liquidation_duration_from_due: DurationInput,
    pub liquidation_cvl: CVLPct,
    pub margin_call_cvl: CVLPct,
    pub initial_cvl: CVLPct,
}
crate::mutation_payload! { TermsTemplateCreatePayload, terms_template: TermsTemplate }

#[derive(InputObject)]
pub(super) struct TermsTemplateUpdateInput {
    pub id: UUID,
    pub annual_rate: AnnualRatePct,
    pub accrual_interval: InterestInterval,
    pub accrual_cycle_interval: InterestInterval,
    pub one_time_fee_rate: OneTimeFeeRatePct,
    pub liquidation_cvl: CVLPct,
    pub duration: DurationInput,
    pub interest_due_duration_from_accrual: DurationInput,
    pub obligation_overdue_duration_from_due: DurationInput,
    pub obligation_liquidation_duration_from_due: DurationInput,
    pub margin_call_cvl: CVLPct,
    pub initial_cvl: CVLPct,
}
crate::mutation_payload! { TermsTemplateUpdatePayload, terms_template: TermsTemplate }
