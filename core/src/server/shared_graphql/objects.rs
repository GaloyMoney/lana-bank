use async_graphql::*;

use crate::primitives::{Satoshis, UsdCents};

use super::primitives::UUID;

#[derive(SimpleObject)]
pub struct BtcBalance {
    pub btc_balance: Satoshis,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaginationKey {
    pub key: UUID,
    pub first: i32,
    pub after: Option<String>,
}
