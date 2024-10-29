use async_graphql::*;

use crate::{
    admin::AdminAuthContext,
    shared_graphql::{customer::Customer, primitives::*},
};
use lava_app::app::LavaApp;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Deposit {
    customer_id: UUID,
    deposit_id: UUID,
    amount: UsdCents,
    reference: String,
    created_at: Timestamp,
}

#[ComplexObject]
impl Deposit {
    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Customer>> {
        let app = ctx.data_unchecked::<LavaApp>();
        let AdminAuthContext { sub } = ctx.data()?;
        let customer = app
            .customers()
            .find_by_id(Some(sub), &self.customer_id)
            .await?;
        Ok(customer.map(Customer::from))
    }
}

impl From<lava_app::deposit::Deposit> for Deposit {
    fn from(deposit: lava_app::deposit::Deposit) -> Self {
        Deposit {
            created_at: deposit.created_at().into(),
            deposit_id: UUID::from(deposit.id),
            customer_id: UUID::from(deposit.customer_id),
            amount: deposit.amount,
            reference: deposit.reference,
        }
    }
}
