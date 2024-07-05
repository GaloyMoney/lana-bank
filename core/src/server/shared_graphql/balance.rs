use async_graphql::*;

use super::objects::{BtcBalance, UsdBalance};

use crate::ledger;

#[derive(SimpleObject)]
struct UnallocatedCollateral {
    settled: BtcBalance,
}

#[derive(SimpleObject)]
struct Checking {
    settled: UsdBalance,
    pending: UsdBalance,
}

#[derive(SimpleObject)]
pub struct UserBalance {
    unallocated_collateral: UnallocatedCollateral,
    checking: Checking,
}

impl From<ledger::user::UserBalance> for UserBalance {
    fn from(balance: ledger::user::UserBalance) -> Self {
        Self {
            unallocated_collateral: UnallocatedCollateral {
                settled: BtcBalance {
                    btc_balance: balance.btc_balance,
                },
            },
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
