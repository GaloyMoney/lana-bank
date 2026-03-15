use async_graphql::{connection::*, *};

use crate::primitives::*;

use super::{
    event_timeline::{self, EventTimelineCursor, EventTimelineEntry},
    terms::*,
};

use lana_app::terms_template::TermsTemplate as DomainTermsTemplate;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct TermsTemplate {
    id: ID,
    terms_template_id: UUID,
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
            terms_template_id: terms.id.into(),
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

    async fn event_history(
        &self,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<EventTimelineCursor, EventTimelineEntry, EmptyFields, EmptyFields>,
    > {
        use es_entity::EsEntity as _;
        event_timeline::events_to_connection(self.entity.events(), first, after)
    }

    async fn user_can_update_terms_template(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .terms_templates()
            .subject_can_update_terms_template(sub, false)
            .await
            .is_ok())
    }
}

#[derive(InputObject)]
pub(super) struct TermsTemplateCreateInput {
    pub name: String,
    pub terms: TermsInput,
}
crate::mutation_payload! { TermsTemplateCreatePayload, terms_template: TermsTemplate }

#[derive(InputObject)]
pub(super) struct TermsTemplateUpdateInput {
    pub terms_template_id: UUID,
    pub terms: TermsInput,
}
crate::mutation_payload! { TermsTemplateUpdatePayload, terms_template: TermsTemplate }
