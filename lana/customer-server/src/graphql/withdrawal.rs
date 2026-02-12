use async_graphql::*;

use crate::primitives::*;

pub(super) use lana_app::deposit::{Withdrawal as DomainWithdrawal, WithdrawalStatus};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub(super) struct Withdrawal {
    id: ID,
    withdrawal_id: UUID,
    account_id: UUID,
    amount: UsdCents,
    status: WithdrawalStatus,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainWithdrawal>,
}

impl From<lana_app::deposit::Withdrawal> for Withdrawal {
    fn from(withdraw: lana_app::deposit::Withdrawal) -> Self {
        Withdrawal {
            id: withdraw.id.to_global_id(),
            created_at: withdraw.created_at().into(),
            account_id: withdraw.deposit_account_id.into(),
            withdrawal_id: UUID::from(withdraw.id),
            amount: withdraw.amount,
            status: withdraw.status(),
            entity: Arc::new(withdraw),
        }
    }
}

#[ComplexObject]
impl Withdrawal {
    async fn reference(&self) -> &str {
        &self.entity.reference
    }
}
