use async_graphql::*;

use crate::{
    ledger,
    server::shared::graphql::objects::{BtcBalance, UsdBalance},
};

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
pub(super) struct UserBalance {
    unallocated_collateral: UnallocatedCollateral,
    checking: Checking,
}

impl From<ledger::user::UserBalance> for UserBalance {
    fn from(balance: ledger::user::UserBalance) -> Self {
        Self {
            unallocated_collateral: UnallocatedCollateral {
                settled: BtcBalance {
                    btc_balance: balance.unallocated_collateral,
                },
            },
            checking: Checking {
                settled: UsdBalance {
                    usd_balance: balance.checking.settled,
                },
                pending: UsdBalance {
                    usd_balance: balance.checking.pending,
                },
            },
        }
    }
}
