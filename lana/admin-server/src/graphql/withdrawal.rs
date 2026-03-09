use async_graphql::*;
use es_entity::Sort;

use crate::primitives::*;

use super::{
    accounting::LedgerTransaction, approval_process::ApprovalProcess,
    deposit_account::DepositAccount, loader::LanaDataLoader, primitives::SortDirection,
};

pub use lana_app::{
    deposit::{
        Withdrawal as DomainWithdrawal, WithdrawalStatus, WithdrawalsCursor,
        WithdrawalsFilters as DomainWithdrawalsFilters,
        WithdrawalsSortBy as DomainWithdrawalsSortBy,
    },
    public_id::PublicId,
};

/// A requested movement of funds out of a deposit account.
#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Withdrawal {
    /// Relay global identifier for this withdrawal.
    id: ID,
    /// Internal UUID for this withdrawal.
    withdrawal_id: UUID,
    /// Internal UUID of the deposit account the funds are drawn from.
    account_id: UUID,
    /// Internal UUID of the approval process governing this withdrawal.
    approval_process_id: UUID,
    /// Amount requested for withdrawal, in USD cents.
    amount: UsdCents,
    /// Current status of the withdrawal.
    status: WithdrawalStatus,
    /// When the withdrawal was created.
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
            status: withdraw.status(),
            entity: Arc::new(withdraw),
        }
    }
}

#[ComplexObject]
impl Withdrawal {
    /// Public identifier assigned to the withdrawal.
    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }

    /// Reference recorded with the withdrawal.
    async fn reference(&self) -> &str {
        &self.entity.reference
    }

    /// Approval process associated with this withdrawal.
    async fn approval_process(&self, ctx: &Context<'_>) -> async_graphql::Result<ApprovalProcess> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let process = loader
            .load_one(self.entity.approval_process_id)
            .await?
            .expect("process not found");
        Ok(process)
    }

    /// Deposit account the withdrawal is drawn from.
    async fn account(&self, ctx: &Context<'_>) -> async_graphql::Result<DepositAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let account = loader
            .load_one(self.entity.deposit_account_id)
            .await?
            .expect("account not found");
        Ok(account)
    }

    /// Ledger transactions produced by this withdrawal.
    async fn ledger_transactions(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<LedgerTransaction>> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let tx_ids = self.entity.ledger_tx_ids();
        let loaded_transactions = loader.load_many(tx_ids.iter().copied()).await?;

        Ok(tx_ids
            .iter()
            .filter_map(|id| loaded_transactions.get(id).cloned())
            .collect())
    }
}

/// Payload published when a withdrawal approval flow finishes.
#[derive(SimpleObject)]
#[graphql(complex)]
pub struct WithdrawalApprovalConcludedPayload {
    /// Final status of the withdrawal after the approval flow finishes.
    pub status: WithdrawalStatus,
    #[graphql(skip)]
    pub withdrawal_id: WithdrawalId,
}

#[ComplexObject]
impl WithdrawalApprovalConcludedPayload {
    /// Withdrawal affected by the completed approval flow.
    async fn withdrawal(&self, ctx: &Context<'_>) -> async_graphql::Result<Withdrawal> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let withdrawal = loader
            .load_one(self.withdrawal_id)
            .await?
            .expect("withdrawal not found");
        Ok(withdrawal)
    }
}

/// Input for initiating a withdrawal.
#[derive(InputObject)]
pub struct WithdrawalInitiateInput {
    /// Internal UUID of the deposit account to withdraw from.
    pub deposit_account_id: UUID,
    /// Amount to withdraw, in USD cents.
    pub amount: UsdCents,
    /// Optional reference to record with the withdrawal.
    pub reference: Option<String>,
}
crate::mutation_payload! { WithdrawalInitiatePayload, withdrawal: Withdrawal }

/// Input for confirming a withdrawal.
#[derive(InputObject)]
pub struct WithdrawalConfirmInput {
    /// Internal UUID of the withdrawal to confirm.
    pub withdrawal_id: UUID,
}
crate::mutation_payload! { WithdrawalConfirmPayload, withdrawal: Withdrawal }

/// Input for cancelling a withdrawal.
#[derive(InputObject)]
pub struct WithdrawalCancelInput {
    /// Internal UUID of the withdrawal to cancel.
    pub withdrawal_id: UUID,
}
crate::mutation_payload! { WithdrawalCancelPayload, withdrawal: Withdrawal }

/// Input for reverting a withdrawal.
#[derive(InputObject)]
pub struct WithdrawalRevertInput {
    /// Internal UUID of the withdrawal to revert.
    pub withdrawal_id: UUID,
}
crate::mutation_payload! { WithdrawalRevertPayload, withdrawal: Withdrawal }

/// Filters that can be applied when listing withdrawals.
#[derive(InputObject)]
pub struct WithdrawalsFilter {
    /// Limit results to a specific withdrawal status.
    pub status: Option<WithdrawalStatus>,
}

/// Fields available when sorting withdrawal lists.
#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WithdrawalsSortBy {
    /// Sort by when the withdrawal was created.
    #[default]
    CreatedAt,
    /// Sort by withdrawal amount.
    Amount,
    /// Sort by the public identifier assigned to the withdrawal.
    PublicId,
}

impl From<WithdrawalsSortBy> for DomainWithdrawalsSortBy {
    fn from(by: WithdrawalsSortBy) -> Self {
        match by {
            WithdrawalsSortBy::CreatedAt => DomainWithdrawalsSortBy::CreatedAt,
            WithdrawalsSortBy::Amount => DomainWithdrawalsSortBy::Amount,
            WithdrawalsSortBy::PublicId => DomainWithdrawalsSortBy::PublicId,
        }
    }
}

/// Sort options for withdrawal lists.
#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct WithdrawalsSort {
    /// Field to sort withdrawals by.
    #[graphql(default)]
    pub by: WithdrawalsSortBy,
    /// Direction to apply to the selected sort field.
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<WithdrawalsSort> for Sort<DomainWithdrawalsSortBy> {
    fn from(sort: WithdrawalsSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}

impl From<WithdrawalsSort> for DomainWithdrawalsSortBy {
    fn from(sort: WithdrawalsSort) -> Self {
        sort.by.into()
    }
}
