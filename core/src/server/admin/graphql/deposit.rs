use async_graphql::*;

use crate::{
    app::LavaApp,
    primitives::UsdCents,
    server::{
        admin::AdminAuthContext,
        shared_graphql::{customer::Customer, primitives::UUID},
    },
};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Deposit {
    customer_id: UUID,
    deposit_id: UUID,
    amount: UsdCents,
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

impl From<crate::deposit::Deposit> for Deposit {
    fn from(deposit: crate::deposit::Deposit) -> Self {
        Deposit {
            deposit_id: UUID::from(deposit.id),
            customer_id: UUID::from(deposit.customer_id),
            amount: deposit.amount,
        }
    }
}

#[derive(InputObject)]
pub struct DepositRecordInput {
    pub customer_id: UUID,
    pub amount: UsdCents,
    pub reference: Option<String>,
}

#[derive(SimpleObject)]
pub struct DepositRecordPayload {
    pub deposit: Deposit,
}

impl From<crate::deposit::Deposit> for DepositRecordPayload {
    fn from(deposit: crate::deposit::Deposit) -> Self {
        Self {
            deposit: Deposit::from(deposit),
        }
    }
}
