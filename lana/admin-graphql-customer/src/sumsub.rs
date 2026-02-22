use async_graphql::*;

use admin_graphql_shared::primitives::*;

#[derive(InputObject)]
pub struct SumsubPermalinkCreateInput {
    pub prospect_id: UUID,
}

#[derive(SimpleObject)]
pub struct SumsubPermalinkCreatePayload {
    pub url: String,
}

#[cfg(feature = "sumsub-testing")]
#[derive(InputObject)]
pub struct SumsubTestApplicantCreateInput {
    pub prospect_id: UUID,
}

#[cfg(feature = "sumsub-testing")]
#[derive(SimpleObject)]
pub struct SumsubTestApplicantCreatePayload {
    pub applicant_id: String,
}
