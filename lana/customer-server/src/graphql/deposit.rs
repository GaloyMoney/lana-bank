use async_graphql::*;

use crate::primitives::*;

pub(super) use lana_app::deposit::{Deposit as DomainDeposit, DepositStatus};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub(super) struct Deposit {
    id: ID,
    deposit_id: UUID,
    account_id: UUID,
    amount: UsdCents,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainDeposit>,
}

impl From<DomainDeposit> for Deposit {
    fn from(deposit: DomainDeposit) -> Self {
        Deposit {
            id: deposit.id.to_global_id(),
            deposit_id: UUID::from(deposit.id),
            account_id: UUID::from(deposit.deposit_account_id),
            amount: deposit.amount,
            created_at: deposit.created_at().into(),

            entity: Arc::new(deposit),
        }
    }
}

#[ComplexObject]
impl Deposit {
    async fn reference(&self) -> &str {
        &self.entity.reference
    }

    async fn status(&self) -> DepositStatus {
        self.entity.status()
    }
}
