use async_graphql::*;

use super::objects::{BtcBalance, UsdBalance};

use crate::ledger;
#[derive(SimpleObject)]
struct Checking {
    settled: UsdBalance,
    pending: UsdBalance,
}

#[derive(SimpleObject)]
pub struct UserBalance {
    checking: Checking,
}

impl From<ledger::customer::CustomerBalance> for UserBalance {
    fn from(balance: ledger::customer::CustomerBalance) -> Self {
        Self {
            checking: Checking {
                settled: UsdBalance {
                    usd_balance: balance.usdt_balance.settled,
                },
                pending: UsdBalance {
                    usd_balance: balance.usdt_balance.pending,
                },
            },
        }
    }
}
