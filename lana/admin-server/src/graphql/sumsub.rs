use async_graphql::*;

use crate::primitives::*;

#[derive(InputObject)]
pub struct ProspectKycLinkCreateInput {
    pub prospect_id: ProspectId,
}

#[derive(SimpleObject)]
pub struct ProspectKycLinkCreatePayload {
    pub url: String,
}
