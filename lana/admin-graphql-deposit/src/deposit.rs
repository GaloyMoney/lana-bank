use async_graphql::*;

use crate::primitives::*;

pub use admin_graphql_shared::deposit::DepositBase;

pub use lana_app::{
    deposit::{
        Deposit as DomainDeposit, DepositAccountsByCreatedAtCursor, DepositStatus,
        DepositsByCreatedAtCursor,
    },
    public_id::PublicId,
};

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
