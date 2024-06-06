use async_graphql::*;

use crate::server::shared::primitives::UsdCents;

#[derive(InputObject)]
pub struct OwnersEquityAddInput {
    pub amount: UsdCents,
    pub reference: String,
}

#[derive(SimpleObject)]
pub struct SuccessPayload {
    pub success: bool,
}

impl From<()> for SuccessPayload {
    fn from(_: ()) -> Self {
        SuccessPayload { success: true }
    }
}
