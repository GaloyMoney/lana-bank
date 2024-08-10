use async_graphql::*;

use super::objects::UsdAmount;

use crate::ledger;
#[derive(SimpleObject)]
struct Checking {
    settled: UsdAmount,
    pending: UsdAmount,
}

#[derive(SimpleObject)]
pub struct CustomerBalance {
    checking: Checking,
}

impl From<ledger::customer::CustomerBalance> for CustomerBalance {
    fn from(balance: ledger::customer::CustomerBalance) -> Self {
        Self {
            checking: Checking {
                settled: UsdAmount {
                    amount: balance.usd_balance.settled,
                },
                pending: UsdAmount {
                    amount: balance.usd_balance.pending,
                },
            },
        }
    }
}
