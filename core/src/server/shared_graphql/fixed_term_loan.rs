use async_graphql::*;

use crate::{app::LavaApp, ledger, server::shared_graphql::primitives::*};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct FixedTermLoan {
    loan_id: UUID,
    user_id: UUID,
    #[graphql(skip)]
    account_ids: ledger::fixed_term_loan::FixedTermLoanAccountIds,
}

#[ComplexObject]
impl FixedTermLoan {
    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<FixedTermLoanBalance> {
        let app = ctx.data_unchecked::<LavaApp>();
        let balance = app
            .ledger()
            .get_fixed_term_loan_balance(self.account_ids)
            .await?;
        Ok(FixedTermLoanBalance::from(balance))
    }
}

impl From<crate::fixed_term_loan::FixedTermLoan> for FixedTermLoan {
    fn from(loan: crate::fixed_term_loan::FixedTermLoan) -> Self {
        FixedTermLoan {
            loan_id: UUID::from(loan.id),
            user_id: UUID::from(loan.user_id),
            account_ids: loan.account_ids,
        }
    }
}

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
