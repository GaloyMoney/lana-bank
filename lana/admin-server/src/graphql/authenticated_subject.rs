use async_graphql::*;

use super::access::User;
use lana_app::{access::user::User as DomainUser, authorization::VisibleNavigationItems};

use crate::primitives::*;

#[derive(SimpleObject)]
#[graphql(name = "Subject", complex)]
pub struct AuthenticatedSubject {
    user: User,
}

#[ComplexObject]
impl AuthenticatedSubject {
    async fn visible_navigation_items(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<VisibleNavigationItems> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let permissions = app.get_visible_nav_items(sub).await?;
        Ok(permissions)
    }

    async fn subject_can_create_customer(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .customers()
            .subject_can_create_customer(sub, false)
            .await
            .is_ok())
    }

    async fn subject_can_create_user(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .access()
            .users()
            .subject_can_create_user(sub, false)
            .await
            .is_ok())
    }

    async fn subject_can_create_terms_template(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .credit()
            .terms_templates()
            .subject_can_create_terms_template(sub, false)
            .await
            .is_ok())
    }
}

impl From<Arc<DomainUser>> for AuthenticatedSubject {
    fn from(entity: Arc<DomainUser>) -> Self {
        Self {
            user: User::from(entity),
        }
    }
}
