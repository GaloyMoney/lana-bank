use async_graphql::*;

use crate::{primitives::UsdCents, server::shared_graphql::primitives::UUID};

#[derive(SimpleObject)]
pub struct Withdrawal {
    customer_id: UUID,
    withdrawal_id: UUID,
    amount: UsdCents,
}

impl From<crate::withdraw::Withdraw> for Withdrawal {
    fn from(withdraw: crate::withdraw::Withdraw) -> Self {
        Withdrawal {
            withdrawal_id: UUID::from(withdraw.id),
            customer_id: UUID::from(withdraw.customer_id),
            amount: withdraw.amount,
        }
    }
}

#[derive(InputObject)]
pub struct WithdrawalInitiateInput {
    pub amount: UsdCents,
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
