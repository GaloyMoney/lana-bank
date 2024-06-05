use async_graphql::*;

use crate::{
    ledger,
    primitives::{Satoshis, UsdCents},
};

#[derive(SimpleObject)]
struct Collateral {
    btc_balance: Satoshis,
}

#[derive(SimpleObject)]
struct LoanOutstanding {
    usd_balance: UsdCents,
}

#[derive(SimpleObject)]
struct InterestIncome {
    usd_balance: UsdCents,
}

#[derive(SimpleObject)]
struct InterestIncurredAndUnpaid {
    usd_balance: UsdCents,
}

#[derive(SimpleObject)]
pub(super) struct FixedTermLoanBalance {
    collateral: Collateral,
    outstanding: LoanOutstanding,
    interest_incurred: InterestIncome,
}

impl From<ledger::fixed_term_loan::FixedTermLoanBalance> for FixedTermLoanBalance {
    fn from(balance: ledger::fixed_term_loan::FixedTermLoanBalance) -> Self {
        Self {
            collateral: Collateral {
                btc_balance: balance.collateral,
            },
            outstanding: LoanOutstanding {
                usd_balance: balance.outstanding,
            },
            interest_incurred: InterestIncome {
                usd_balance: balance.interest_incurred,
            },
        }
    }
}
