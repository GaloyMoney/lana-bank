use async_graphql::{Context, Object};

use crate::{LanaApp, primitives::*};

use super::{authenticated_subject::*, credit_facility::*, price::*};

pub struct Query;

#[Object]
impl Query {
    async fn me(&self, ctx: &Context<'_>) -> async_graphql::Result<AuthenticatedSubject> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let customer = app.get_customer_with_activity(sub).await?;
        Ok(AuthenticatedSubject::from(customer))
    }

    async fn credit_facility(
        &self,
        ctx: &Context<'_>,
        id: UUID,
    ) -> async_graphql::Result<Option<CreditFacility>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);

        Ok(app
            .credit()
            .for_subject(sub)?
            .find_by_id(id)
            .await?
            .map(CreditFacility::from))
    }

    async fn realtime_price(&self, ctx: &Context<'_>) -> async_graphql::Result<RealtimePrice> {
        let app = ctx.data_unchecked::<LanaApp>();
        let usd_cents_per_btc = app.price().usd_cents_per_btc().await?;
        Ok(usd_cents_per_btc.into())
    }
}
