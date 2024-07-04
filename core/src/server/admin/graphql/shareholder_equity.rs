use async_graphql::*;

use crate::server::shared_graphql::primitives::SignedUsdCents;

#[derive(InputObject)]
pub struct ShareholderEquityAddInput {
    pub amount: SignedUsdCents,
    pub reference: String,
}
