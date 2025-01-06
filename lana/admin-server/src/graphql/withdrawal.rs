use async_graphql::*;

use crate::primitives::*;

use super::{
    approval_process::ApprovalProcess, deposit_account::DepositAccount, loader::LanaDataLoader,
};

pub use lana_app::deposit::{
    Withdrawal as DomainWithdrawal, WithdrawalStatus, WithdrawalsByCreatedAtCursor,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Withdrawal {
    id: ID,
    withdrawal_id: UUID,
    account_id: UUID,
    approval_process_id: UUID,
    amount: UsdCents,
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
            .deposits()
            .ensure_up_to_date_status(&self.entity)
            .await?
            .map(|w| w.status())
            .unwrap_or_else(|| self.entity.status()))
    }

    async fn approval_process(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcess> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let process = loader
            .load_one(self.entity.approval_process_id)
            .await?
            .expect("process not found");
        Ok(process)
    }

    async fn account(&self, ctx: &Context<'_>) -> async_graphql::Result<DepositAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let account = loader
            .load_one(self.entity.deposit_account_id)
            .await?
            .expect("account not found");
        Ok(account)
    }

    // async fn subject_can_confirm(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
    //     let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
    //     Ok(app
    //         .withdrawals()
    //         .subject_can_confirm(sub, false)
    //         .await
    //         .is_ok())
    // }

    // async fn subject_can_cancel(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
    //     let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
    //     Ok(app
    //         .withdrawals()
    //         .subject_can_cancel(sub, false)
    //         .await
    //         .is_ok())
    // }
}

#[derive(InputObject)]
pub struct WithdrawalInitiateInput {
    pub deposit_account_id: UUID,
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
