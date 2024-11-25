use async_graphql::*;

use crate::primitives::*;

use super::{approval_process::ApprovalProcess, customer::Customer, loader::LanaDataLoader};

pub use lana_app::withdrawal::{
    Withdrawal as DomainWithdrawal, WithdrawalStatus, WithdrawalsByCreatedAtCursor,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Withdrawal {
    id: ID,
    withdrawal_id: UUID,
    customer_id: UUID,
    approval_process_id: UUID,
    amount: UsdCents,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainWithdrawal>,
}

impl From<lana_app::withdrawal::Withdrawal> for Withdrawal {
    fn from(withdraw: lana_app::withdrawal::Withdrawal) -> Self {
        Withdrawal {
            id: withdraw.id.to_global_id(),
            created_at: withdraw.created_at().into(),
            withdrawal_id: UUID::from(withdraw.id),
            customer_id: UUID::from(withdraw.customer_id),
            approval_process_id: UUID::from(withdraw.approval_process_id),
            amount: withdraw.amount,
            entity: Arc::new(withdraw),
        }
    }
}

#[ComplexObject]
impl Withdrawal {
    async fn reference(&self) -> &str {
        &self.entity.reference
    }

    async fn status(&self, ctx: &Context<'_>) -> async_graphql::Result<WithdrawalStatus> {
        let (app, _) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .withdrawals()
            .ensure_up_to_date_status(&self.entity)
            .await?
            .map(|w| w.status())
            .unwrap_or_else(|| self.entity.status()))
    }

    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let customer = loader
            .load_one(self.entity.customer_id)
            .await?
            .expect("policy not found");
        Ok(customer)
    }

    async fn approval_process(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcess> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let process = loader
            .load_one(self.entity.approval_process_id)
            .await?
            .expect("process not found");
        Ok(process)
    }

    async fn subject_can_confirm(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .withdrawals()
            .subject_can_confirm(sub, false)
            .await
            .is_ok())
    }

    async fn subject_can_cancel(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        Ok(app
            .withdrawals()
            .subject_can_cancel(sub, false)
            .await
            .is_ok())
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
