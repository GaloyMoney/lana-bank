use crate::primitives::UsdCents;

use async_graphql::*;

#[derive(SimpleObject)]
pub struct UsdBalance {
    pub usd_balance: UsdCents,
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
