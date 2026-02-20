use async_graphql::*;

use super::primitives::*;

pub use lana_app::{
    deposit::{
        Deposit as DomainDeposit, DepositAccountsByCreatedAtCursor, DepositStatus,
        DepositsByCreatedAtCursor,
    },
    public_id::PublicId,
};

#[derive(SimpleObject, Clone)]
#[graphql(name = "Deposit", complex)]
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
