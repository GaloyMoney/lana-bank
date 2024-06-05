use async_graphql::*;

use crate::{
    app::LavaApp,
    ledger::user::UserLedgerAccountIds,
    primitives::{Satoshis, UsdCents},
    server::shared::primitives::UUID,
};

use super::user_balance::*;

#[derive(InputObject)]
pub struct UserCreateInput {
    pub bitfinex_username: String,
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct User {
    user_id: UUID,
    bitfinex_username: String,
    #[graphql(skip)]
    account_ids: UserLedgerAccountIds,
}

#[ComplexObject]
impl User {
    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<UserBalance> {
        let app = ctx.data_unchecked::<LavaApp>();
        let balance = app.ledger().get_user_balance(self.account_ids).await?;
        Ok(UserBalance::from(balance))
    }
}

impl From<crate::user::User> for User {
    fn from(user: crate::user::User) -> Self {
        User {
            user_id: UUID::from(user.id),
            bitfinex_username: user.bitfinex_username,
            account_ids: user.account_ids,
        }
    }
}

#[derive(InputObject)]
pub struct UserPledgeCollateralInput {
    pub user_id: UUID,
    pub amount: Satoshis,
    pub reference: String,
}

#[derive(SimpleObject)]
pub struct UserPledgeCollateralPayload {
    pub user: User,
}

impl From<crate::user::User> for UserPledgeCollateralPayload {
    fn from(user: crate::user::User) -> Self {
        Self {
            user: User::from(user),
        }
    }
}

#[derive(InputObject)]
pub struct UserDepositInput {
    pub user_id: UUID,
    pub amount: UsdCents,
    pub reference: String,
}

#[derive(SimpleObject)]
pub struct UserDepositPayload {
    pub user: User,
}

impl From<crate::user::User> for UserDepositPayload {
    fn from(user: crate::user::User) -> Self {
        Self {
            user: User::from(user),
        }
    }
}

#[derive(SimpleObject)]
pub struct Withdrawal {
    withdrawal_id: UUID,
    user_id: UUID,
    amount: UsdCents,
}

impl From<crate::withdraw::Withdraw> for Withdrawal {
    fn from(withdraw: crate::withdraw::Withdraw) -> Self {
        Withdrawal {
            withdrawal_id: UUID::from(withdraw.id),
            user_id: UUID::from(withdraw.user_id),
            amount: withdraw.amount,
        }
    }
}

#[derive(InputObject)]
pub struct WithdrawalSettleInput {
    pub withdrawal_id: UUID,
    pub reference: String,
}

#[derive(SimpleObject)]
pub struct WithdrawalSettlePayload {
    pub withdrawal: Withdrawal,
}

impl From<crate::withdraw::Withdraw> for WithdrawalSettlePayload {
    fn from(withdrawal: crate::withdraw::Withdraw) -> Self {
        Self {
            withdrawal: Withdrawal::from(withdrawal),
        }
    }
}
