use async_graphql::*;

use crate::primitives::*;

pub use admin_graphql_shared::deposit::WithdrawalBase;

pub use lana_app::{
    deposit::{Withdrawal as DomainWithdrawal, WithdrawalStatus, WithdrawalsByCreatedAtCursor},
    public_id::PublicId,
};

#[derive(InputObject)]
pub struct WithdrawalInitiateInput {
    pub deposit_account_id: UUID,
    pub amount: UsdCents,
    pub reference: Option<String>,
}

#[derive(InputObject)]
pub struct WithdrawalConfirmInput {
    pub withdrawal_id: UUID,
}

#[derive(InputObject)]
pub struct WithdrawalCancelInput {
    pub withdrawal_id: UUID,
}

#[derive(InputObject)]
pub struct WithdrawalRevertInput {
    pub withdrawal_id: UUID,
}
