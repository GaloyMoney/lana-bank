use async_graphql::*;

use crate::primitives::*;

#[derive(InputObject)]
pub struct SumsubPermalinkCreateInput {
    pub customer_id: UUID,
}

#[derive(SimpleObject)]
pub struct SumsubPermalinkCreatePayload {
    pub url: String,
}

impl From<lana_app::kyc::PermalinkResponse> for SumsubPermalinkCreatePayload {
    fn from(resp: lana_app::kyc::PermalinkResponse) -> Self {
        Self { url: resp.url }
    }
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
