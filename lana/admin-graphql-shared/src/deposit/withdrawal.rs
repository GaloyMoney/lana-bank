use async_graphql::*;

use super::primitives::*;

pub use lana_app::{
    deposit::{Withdrawal as DomainWithdrawal, WithdrawalStatus, WithdrawalsByCreatedAtCursor},
    public_id::PublicId,
};

#[derive(SimpleObject, Clone)]
#[graphql(name = "Withdrawal", complex)]
pub struct WithdrawalBase {
    id: ID,
    withdrawal_id: UUID,
    account_id: UUID,
    approval_process_id: UUID,
    amount: UsdCents,
    status: WithdrawalStatus,
    created_at: Timestamp,

    #[graphql(skip)]
    pub entity: Arc<DomainWithdrawal>,
}

impl From<DomainWithdrawal> for WithdrawalBase {
    fn from(withdraw: DomainWithdrawal) -> Self {
        Self {
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
impl WithdrawalBase {
    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }

    async fn reference(&self) -> &str {
        &self.entity.reference
    }

    async fn approval_process(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<crate::governance::ApprovalProcess> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let processes: std::collections::HashMap<_, crate::governance::ApprovalProcess> = app
            .governance()
            .find_all_approval_processes(&[self.entity.approval_process_id])
            .await?;
        Ok(processes
            .into_values()
            .next()
            .expect("withdrawal must have an approval process"))
    }

    async fn ledger_transactions(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<crate::accounting::LedgerTransaction>> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let tx_ids = self.entity.ledger_tx_ids();
        let loaded_transactions: std::collections::HashMap<
            _,
            crate::accounting::LedgerTransaction,
        > = app
            .accounting()
            .ledger_transactions()
            .find_all(&tx_ids)
            .await?;

        Ok(tx_ids
            .iter()
            .filter_map(|id| loaded_transactions.get(id).cloned())
            .collect())
    }
}
