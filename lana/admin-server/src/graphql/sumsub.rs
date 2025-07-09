use async_graphql::*;

use crate::primitives::*;

#[derive(SimpleObject)]
pub struct SumsubTokenCreatePayload {
    pub token: String,
}

#[derive(InputObject)]
pub struct SumsubPermalinkCreateInput {
    pub customer_id: UUID,
}

#[derive(SimpleObject)]
pub struct SumsubPermalinkCreatePayload {
    pub url: String,
}

#[cfg(feature = "sumsub-testing")]
#[derive(InputObject)]
pub struct SumsubTestApplicantCreateInput {
    pub customer_id: UUID,
}

#[cfg(feature = "sumsub-testing")]
#[derive(SimpleObject)]
pub struct SumsubTestApplicantCreatePayload {
    pub applicant_id: String,
}
