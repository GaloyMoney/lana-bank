use async_graphql::*;

use crate::{primitives::UsdCents, server::shared::primitives::UUID};

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
