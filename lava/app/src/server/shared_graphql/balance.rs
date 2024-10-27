use async_graphql::*;

use crate::ledger;

use super::primitives::UsdCents;

#[derive(SimpleObject)]
struct Checking {
    settled: UsdCents,
    pending: UsdCents,
}

#[derive(SimpleObject)]
pub struct CustomerBalance {
    checking: Checking,
}

impl From<ledger::customer::CustomerBalance> for CustomerBalance {
    fn from(balance: ledger::customer::CustomerBalance) -> Self {
        Self {
            checking: Checking {
                settled: balance.usd_balance.settled,
                pending: balance.usd_balance.pending,
            },
        }
    }
}
