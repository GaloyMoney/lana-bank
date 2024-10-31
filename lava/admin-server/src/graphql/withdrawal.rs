use async_graphql::*;

use crate::primitives::*;

use super::{approval_process::ApprovalProcess, customer::Customer, loader::LavaDataLoader};

pub use lava_app::withdraw::{
    Withdraw as DomainWithdrawal, WithdrawByCreatedAtCursor, WithdrawalStatus,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Withdrawal {
    id: ID,
    withdrawal_id: UUID,
    customer_id: UUID,
    approval_process_id: UUID,
    amount: UsdCents,
    status: WithdrawalStatus,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainWithdrawal>,
}

impl From<lava_app::withdraw::Withdraw> for Withdrawal {
    fn from(withdraw: lava_app::withdraw::Withdraw) -> Self {
        Withdrawal {
            id: withdraw.id.to_global_id(),
            created_at: withdraw.created_at().into(),
            withdrawal_id: UUID::from(withdraw.id),
            customer_id: UUID::from(withdraw.customer_id),
            approval_process_id: UUID::from(withdraw.approval_process_id),
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

    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let loader = ctx.data_unchecked::<LavaDataLoader>();
        let customer = loader
            .load_one(self.entity.customer_id)
            .await?
            .expect("policy not found");
        Ok(customer)
    }

    async fn approval_process(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcess> {
        let loader = ctx.data_unchecked::<LavaDataLoader>();
        let process = loader
            .load_one(self.entity.approval_process_id)
            .await?
            .expect("process not found");
        Ok(process)
    }

    async fn user_can_confirm(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .withdraws()
            .subject_can_confirm(sub, false)
            .await
            .is_ok())
    }

    async fn user_can_cancel(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app.withdraws().subject_can_cancel(sub, false).await.is_ok())
    }
}

#[derive(InputObject)]
pub struct WithdrawalInitiateInput {
    pub customer_id: UUID,
    pub amount: UsdCents,
    pub reference: Option<String>,
}
crate::mutation_payload! { WithdrawalInitiatePayload, withdrawal: Withdrawal }

#[derive(InputObject)]
pub struct WithdrawalConfirmInput {
    pub withdrawal_id: UUID,
}
crate::mutation_payload! { WithdrawalConfirmPayload, withdrawal: Withdrawal }

#[derive(InputObject)]
pub struct WithdrawalCancelInput {
    pub withdrawal_id: UUID,
}
crate::mutation_payload! { WithdrawalCancelPayload, withdrawal: Withdrawal }
