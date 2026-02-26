use async_graphql::{Context, Object};

use admin_graphql_shared::primitives::*;

use super::*;

#[derive(Default)]
pub struct SessionQuery;

#[Object]
impl SessionQuery {
    async fn me(&self, ctx: &Context<'_>) -> async_graphql::Result<MeUser> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let user = Arc::new(app.access().users().find_for_subject(sub).await?);
        Ok(MeUser::from(user))
    }

    async fn dashboard(&self, ctx: &Context<'_>) -> async_graphql::Result<Dashboard> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let dashboard = app.dashboard().load(sub).await?;
        Ok(Dashboard::from(dashboard))
    }
}

#[derive(Default)]
pub struct SessionMutation;

#[Object]
impl SessionMutation {
    async fn sumsub_permalink_create(
        &self,
        ctx: &Context<'_>,
        input: SumsubPermalinkCreateInput,
    ) -> async_graphql::Result<SumsubPermalinkCreatePayload> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let permalink = app
            .customer_kyc()
            .create_verification_link(
                sub,
                lana_app::primitives::ProspectId::from(input.prospect_id),
            )
            .await?;
        Ok(SumsubPermalinkCreatePayload { url: permalink.url })
    }

    /// ⚠️ TEST ONLY: Creates a complete test applicant for Sumsub integration testing.
    /// This method is behind a compilation flag and should only be used in test environments.
    #[cfg(feature = "sumsub-testing")]
    async fn sumsub_test_applicant_create(
        &self,
        ctx: &Context<'_>,
        input: SumsubTestApplicantCreateInput,
    ) -> async_graphql::Result<SumsubTestApplicantCreatePayload> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let applicant_id = app
            .customer_kyc()
            .create_complete_test_applicant(lana_app::primitives::ProspectId::from(
                input.prospect_id,
            ))
            .await?;
        Ok(SumsubTestApplicantCreatePayload { applicant_id })
    }
}
