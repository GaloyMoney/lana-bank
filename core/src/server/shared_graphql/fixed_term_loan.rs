use async_graphql::*;

use crate::{
    app::LavaApp,
    ledger,
    primitives::UserId,
    server::shared_graphql::{primitives::*, user::User},
};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct FixedTermLoan {
    loan_id: UUID,
    #[graphql(skip)]
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

    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        let app = ctx.data_unchecked::<LavaApp>();
        let user = app.users().find_by_id(UserId::from(&self.user_id)).await?;

        match user {
            Some(user) => Ok(User::from(user)),
            None => panic!("user not found for a loan. should not be possible"),
        }
    }
}

impl From<crate::fixed_term_loan::FixedTermLoan> for FixedTermLoan {
    fn from(loan: crate::fixed_term_loan::FixedTermLoan) -> Self {
        FixedTermLoan {
            user_id: UUID::from(loan.user_id),
            loan_id: UUID::from(loan.id),
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
