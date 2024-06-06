use async_graphql::*;

use crate::{
    primitives::UsdCents,
    server::shared_graphql::{
        primitives::UUID,
        user::{User, Withdrawal},
    },
};

#[derive(InputObject)]
pub struct UserCreateInput {
    pub bitfinex_username: String,
}

#[derive(SimpleObject)]
pub struct UserCreatePayload {
    user: User,
}

impl From<crate::user::User> for UserCreatePayload {
    fn from(user: crate::user::User) -> Self {
        Self {
            user: User::from(user),
        }
    }
}

#[derive(InputObject)]
pub struct WithdrawalInitiateInput {
    pub user_id: UUID,
    pub amount: UsdCents,
    pub destination: String,
    pub reference: String,
}

#[derive(SimpleObject)]
pub struct WithdrawalInitiatePayload {
    pub withdrawal: Withdrawal,
}

impl From<crate::withdraw::Withdraw> for WithdrawalInitiatePayload {
    fn from(withdrawal: crate::withdraw::Withdraw) -> Self {
        Self {
            withdrawal: Withdrawal::from(withdrawal),
        }
    }
}
