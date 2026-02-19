use async_graphql::*;

use crate::primitives::*;

pub use lana_app::{
    deposit::{
        Deposit as DomainDeposit, DepositAccountsByCreatedAtCursor, DepositStatus,
        DepositsByCreatedAtCursor,
    },
    public_id::PublicId,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct DepositBase {
    id: ID,
    deposit_id: UUID,
    account_id: UUID,
    amount: UsdCents,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainDeposit>,
}

impl From<DomainDeposit> for DepositBase {
    fn from(deposit: DomainDeposit) -> Self {
        Self {
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
impl DepositBase {
    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }

    async fn reference(&self) -> &str {
        &self.entity.reference
    }

    async fn status(&self) -> DepositStatus {
        self.entity.status()
    }
}

#[derive(InputObject)]
pub struct DepositRecordInput {
    pub deposit_account_id: UUID,
    pub amount: UsdCents,
    pub reference: Option<String>,
}

#[derive(InputObject)]
pub struct DepositAccountCreateInput {
    pub customer_id: UUID,
}

#[derive(InputObject)]
pub struct DepositRevertInput {
    pub deposit_id: UUID,
}

#[derive(InputObject)]
pub struct DepositAccountFreezeInput {
    pub deposit_account_id: UUID,
}

#[derive(InputObject)]
pub struct DepositAccountUnfreezeInput {
    pub deposit_account_id: UUID,
}

#[derive(InputObject)]
pub struct DepositAccountCloseInput {
    pub deposit_account_id: UUID,
}
