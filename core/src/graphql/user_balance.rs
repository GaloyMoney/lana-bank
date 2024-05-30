use async_graphql::*;

use crate::ledger;

use super::objects::{BtcBalance, UsdBalance};

#[derive(SimpleObject)]
struct UnallocatedCollateral {
    settled: BtcBalance,
}

#[derive(SimpleObject)]
struct Checking {
    settled: UsdBalance,
    encumbrance: UsdBalance,
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
                encumbrance: UsdBalance {
                    usd_balance: balance.checking.encumbrance,
                },
            },
        }
    }
}
