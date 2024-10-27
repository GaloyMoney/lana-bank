use async_graphql::*;

use crate::server::shared_graphql::primitives::UsdCents;

#[derive(InputObject)]
pub struct ShareholderEquityAddInput {
    pub amount: UsdCents,
    pub reference: String,
}
