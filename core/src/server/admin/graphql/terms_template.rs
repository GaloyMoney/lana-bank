use async_graphql::*;

use crate::{
    loan::{AnnualRatePct, CVLPct},
    server::shared_graphql::{
        convert::ToGlobalId,
        primitives::UUID,
        terms::{DurationInput, InterestInterval, TermValues},
    },
};

#[derive(InputObject)]
pub(super) struct CreateTermsTemplateInput {
    pub name: String,
    pub annual_rate: AnnualRatePct,
    pub interval: InterestInterval,
    pub duration: DurationInput,
    pub liquidation_cvl: CVLPct,
    pub margin_call_cvl: CVLPct,
    pub initial_cvl: CVLPct,
}

#[derive(SimpleObject)]
pub struct CreateTermsTemplatePayload {
    pub terms_template: TermsTemplate,
}

impl From<crate::terms_template::TermsTemplate> for CreateTermsTemplatePayload {
    fn from(terms_template: crate::terms_template::TermsTemplate) -> Self {
        Self {
            terms_template: terms_template.into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct TermsTemplate {
    id: ID,
    name: String,
    terms_id: UUID,
    values: TermValues,
}

impl From<crate::terms_template::TermsTemplate> for TermsTemplate {
    fn from(terms: crate::terms_template::TermsTemplate) -> Self {
        Self {
            id: terms.id.to_global_id(),
            name: terms.name.into(),
            terms_id: terms.id.into(),
            values: terms.values.into(),
        }
    }
}
