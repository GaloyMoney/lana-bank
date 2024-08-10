use crate::primitives::UsdCents;

use async_graphql::*;

#[derive(SimpleObject)]
pub struct UsdAmount {
    pub amount: UsdCents,
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
