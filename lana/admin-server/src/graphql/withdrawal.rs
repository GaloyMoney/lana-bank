use async_graphql::*;

use crate::primitives::*;

use super::{
    accounting::LedgerTransaction, approval_process::ApprovalProcess,
    deposit_account::DepositAccount, loader::LanaDataLoader,
};

pub use admin_graphql_deposit::{
    DomainWithdrawal, WithdrawalBase, WithdrawalCancelInput, WithdrawalConfirmInput,
    WithdrawalInitiateInput, WithdrawalRevertInput, WithdrawalsByCreatedAtCursor,
};

// ===== Withdrawal =====

#[derive(Clone)]
pub(super) struct WithdrawalCrossDomain {
    entity: Arc<DomainWithdrawal>,
}

#[Object]
impl WithdrawalCrossDomain {
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

#[derive(MergedObject, Clone)]
#[graphql(name = "Withdrawal")]
pub struct Withdrawal(pub WithdrawalBase, WithdrawalCrossDomain);

impl From<DomainWithdrawal> for Withdrawal {
    fn from(withdrawal: DomainWithdrawal) -> Self {
        let base = WithdrawalBase::from(withdrawal);
        let cross = WithdrawalCrossDomain {
            entity: base.entity.clone(),
        };
        Self(base, cross)
    }
}

impl std::ops::Deref for Withdrawal {
    type Target = WithdrawalBase;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

crate::mutation_payload! { WithdrawalInitiatePayload, withdrawal: Withdrawal }
crate::mutation_payload! { WithdrawalConfirmPayload, withdrawal: Withdrawal }
crate::mutation_payload! { WithdrawalCancelPayload, withdrawal: Withdrawal }
crate::mutation_payload! { WithdrawalRevertPayload, withdrawal: Withdrawal }
