use async_graphql::*;

use super::objects::UsdBalance;

use crate::ledger;
#[derive(SimpleObject)]
struct Checking {
    settled: UsdBalance,
    pending: UsdBalance,
}

#[derive(SimpleObject)]
pub struct CustomerBalance {
    checking: Checking,
}

impl From<ledger::customer::CustomerBalance> for CustomerBalance {
    fn from(balance: ledger::customer::CustomerBalance) -> Self {
        Self {
            checking: Checking {
                settled: UsdBalance {
                    usd_balance: balance.usd_balance.settled,
                },
                pending: UsdBalance {
                    usd_balance: balance.usd_balance.pending,
                },
            },
        }
    }
}
