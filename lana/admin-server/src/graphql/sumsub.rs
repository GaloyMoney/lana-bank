use async_graphql::*;

use crate::primitives::*;

#[derive(InputObject)]
pub struct ProspectKycLinkCreateInput {
    pub prospect_id: UUID,
}

#[derive(SimpleObject)]
pub struct ProspectKycLinkCreatePayload {
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
