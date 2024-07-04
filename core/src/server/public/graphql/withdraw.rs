use async_graphql::*;

use crate::server::shared_graphql::primitives::{SignedUsdCents, UUID};

#[derive(SimpleObject)]
pub struct Withdrawal {
    user_id: UUID,
    withdrawal_id: UUID,
    amount: SignedUsdCents,
}

impl From<crate::withdraw::Withdraw> for Withdrawal {
    fn from(withdraw: crate::withdraw::Withdraw) -> Self {
        Withdrawal {
            withdrawal_id: UUID::from(withdraw.id),
            user_id: UUID::from(withdraw.user_id),
            amount: withdraw.amount.into(),
        }
    }
}

#[derive(InputObject)]
pub struct WithdrawalInitiateInput {
    pub amount: SignedUsdCents,
    pub destination: String,
    pub reference: Option<String>,
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
