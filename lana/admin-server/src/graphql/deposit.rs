use async_graphql::*;

use crate::primitives::*;

pub use lana_app::deposit::{Deposit as DomainDeposit, DepositsByCreatedAtCursor};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Deposit {
    id: ID,
    deposit_id: UUID,
    deposit_account_id: UUID,
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
            deposit_account_id: UUID::from(deposit.deposit_account_id),
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
}

#[derive(InputObject)]
pub struct DepositRecordInput {
    pub deposit_account_id: UUID,
    pub amount: UsdCents,
    pub reference: Option<String>,
}
crate::mutation_payload! { DepositRecordPayload, deposit: Deposit }
