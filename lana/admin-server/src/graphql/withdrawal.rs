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

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Withdrawal {
    id: ID,
    withdrawal_id: UUID,
    account_id: UUID,
    approval_process_id: UUID,
    amount: UsdCents,
    status: WithdrawalStatus,
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
    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }

    async fn reference(&self) -> &str {
        &self.entity.reference
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

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct WithdrawalApprovalConcludedPayload {
    pub status: WithdrawalStatus,
    #[graphql(skip)]
    pub withdrawal_id: WithdrawalId,
}

#[ComplexObject]
impl WithdrawalApprovalConcludedPayload {
    async fn withdrawal(&self, ctx: &Context<'_>) -> async_graphql::Result<Withdrawal> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let withdrawal = loader
            .load_one(self.withdrawal_id)
            .await?
            .expect("withdrawal not found");
        Ok(withdrawal)
    }
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

#[derive(InputObject)]
pub struct WithdrawalRevertInput {
    pub withdrawal_id: UUID,
}
crate::mutation_payload! { WithdrawalRevertPayload, withdrawal: Withdrawal }

#[derive(InputObject)]
pub struct WithdrawalsFilter {
    pub status: Option<WithdrawalStatus>,
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WithdrawalsSortBy {
    #[default]
    CreatedAt,
    Amount,
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

#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct WithdrawalsSort {
    #[graphql(default)]
    pub by: WithdrawalsSortBy,
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
