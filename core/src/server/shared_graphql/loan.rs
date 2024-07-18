use async_graphql::*;

use crate::primitives;

use crate::{
    app::LavaApp,
    ledger,
    primitives::UserId,
    server::shared_graphql::{primitives::*, user::User},
};

use super::convert::ToGlobalId;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "crate::primitives::LoanStatus")]
pub enum LoanStatus {
    New,
    Active,
    Closed,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Loan {
    id: ID,
    loan_id: UUID,
    start_date: Timestamp,
    #[graphql(skip)]
    user_id: UUID,
    #[graphql(skip)]
    account_ids: crate::ledger::loan::LoanAccountIds,
    status: LoanStatus,
}

#[ComplexObject]
impl Loan {
    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<LoanBalance> {
        let app = ctx.data_unchecked::<LavaApp>();
        let balance = app.ledger().get_loan_balance(self.account_ids).await?;
        Ok(LoanBalance::from(balance))
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
pub(super) struct LoanBalance {
    collateral: Collateral,
    outstanding: LoanOutstanding,
    interest_incurred: InterestIncome,
}

impl From<ledger::loan::LoanBalance> for LoanBalance {
    fn from(balance: ledger::loan::LoanBalance) -> Self {
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

impl ToGlobalId for crate::primitives::LoanId {
    fn to_global_id(&self) -> async_graphql::types::ID {
        async_graphql::types::ID::from(format!("loan:{}", self))
    }
}

impl From<primitives::LoanStatus> for LoanStatus {
    fn from(level: primitives::LoanStatus) -> Self {
        match level {
            primitives::LoanStatus::Active => LoanStatus::Active,
            primitives::LoanStatus::Inactive => LoanStatus::Inactive,
        }
    }
}

impl From<crate::loan::Loan> for Loan {
    fn from(loan: crate::loan::Loan) -> Self {
        Loan {
            id: loan.id.to_global_id(),
            loan_id: UUID::from(loan.id),
            user_id: UUID::from(loan.user_id),
            account_ids: loan.account_ids,
            start_date: Timestamp::from(loan.start_date),
            status: loan.status().into(),
        }
    }
}
