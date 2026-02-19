use async_graphql::*;

use crate::primitives::*;

pub use lana_app::{
    deposit::{Withdrawal as DomainWithdrawal, WithdrawalStatus, WithdrawalsByCreatedAtCursor},
    public_id::PublicId,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct WithdrawalBase {
    id: ID,
    withdrawal_id: UUID,
    account_id: UUID,
    approval_process_id: UUID,
    amount: UsdCents,
    status: WithdrawalStatus,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainWithdrawal>,
}

impl From<DomainWithdrawal> for WithdrawalBase {
    fn from(withdraw: DomainWithdrawal) -> Self {
        Self {
            id: withdraw.id.to_global_id(),
            created_at: withdraw.created_at().into(),
            account_id: withdraw.deposit_account_id.into(),
            withdrawal_id: UUID::from(withdraw.id),
            approval_process_id: UUID::from(withdraw.approval_process_id),
            amount: withdraw.amount,
            status: withdraw.status(),
            entity: Arc::new(withdraw),
        }
    }
}

#[ComplexObject]
impl WithdrawalBase {
    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }

    async fn reference(&self) -> &str {
        &self.entity.reference
    }
}

#[derive(InputObject)]
pub struct WithdrawalInitiateInput {
    pub deposit_account_id: UUID,
    pub amount: UsdCents,
    pub reference: Option<String>,
}

#[derive(InputObject)]
pub struct WithdrawalConfirmInput {
    pub withdrawal_id: UUID,
}

#[derive(InputObject)]
pub struct WithdrawalCancelInput {
    pub withdrawal_id: UUID,
}

#[derive(InputObject)]
pub struct WithdrawalRevertInput {
    pub withdrawal_id: UUID,
}
