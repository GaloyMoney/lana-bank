use async_graphql::*;

use crate::{
    app::LavaApp,
    primitives::UsdCents,
    server::{
        admin::AdminAuthContext,
        shared_graphql::{customer::Customer, primitives::*},
    },
    withdraw::WithdrawalStatus,
};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Withdrawal {
    customer_id: UUID,
    withdrawal_id: UUID,
    amount: UsdCents,
    status: WithdrawalStatus,
    reference: String,
    created_at: Timestamp,
}

#[ComplexObject]
impl Withdrawal {
    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Customer>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let customer = app
            .customers()
            .find_by_id(Some(sub), &self.customer_id)
            .await?;
        Ok(customer.map(Customer::from))
    }

    async fn user_can_confirm(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        Ok(app.withdraws().user_can_confirm(sub, false).await.is_ok())
    }

    async fn user_can_cancel(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        Ok(app.withdraws().user_can_cancel(sub, false).await.is_ok())
    }
}

impl From<crate::withdraw::Withdraw> for Withdrawal {
    fn from(withdraw: crate::withdraw::Withdraw) -> Self {
        Withdrawal {
            created_at: withdraw.created_at().into(),
            withdrawal_id: UUID::from(withdraw.id),
            customer_id: UUID::from(withdraw.customer_id),
            amount: withdraw.amount,
            status: withdraw.status(),
            reference: withdraw.reference,
        }
    }
}
