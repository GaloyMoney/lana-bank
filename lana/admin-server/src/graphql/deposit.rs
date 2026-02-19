use async_graphql::*;

use crate::{graphql::accounting::LedgerTransaction, primitives::*};

use super::loader::LanaDataLoader;

pub use admin_graphql_deposit::{
    DepositAccountCloseInput, DepositAccountCreateInput, DepositAccountFreezeInput,
    DepositAccountUnfreezeInput, DepositAccountsByCreatedAtCursor, DepositBase, DepositRecordInput,
    DepositRevertInput, DepositsByCreatedAtCursor, DomainDeposit,
};

pub use super::deposit_account::DepositAccount;

// ===== Deposit =====

#[derive(Clone)]
pub(super) struct DepositCrossDomain {
    entity: Arc<DomainDeposit>,
}

#[Object]
impl DepositCrossDomain {
    async fn account(&self, ctx: &Context<'_>) -> async_graphql::Result<DepositAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let account = loader
            .load_one(self.entity.deposit_account_id)
            .await?
            .expect("process not found");
        Ok(account)
    }

    async fn ledger_transactions(&self, ctx: &Context<'_>) -> Result<Vec<LedgerTransaction>> {
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
#[graphql(name = "Deposit")]
pub struct Deposit(pub DepositBase, DepositCrossDomain);

impl From<DomainDeposit> for Deposit {
    fn from(deposit: DomainDeposit) -> Self {
        let base = DepositBase::from(deposit);
        let cross = DepositCrossDomain {
            entity: base.entity.clone(),
        };
        Self(base, cross)
    }
}

impl std::ops::Deref for Deposit {
    type Target = DepositBase;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

crate::mutation_payload! { DepositRecordPayload, deposit: Deposit }
crate::mutation_payload! { DepositRevertPayload, deposit: Deposit }
crate::mutation_payload! { DepositAccountCreatePayload, account: DepositAccount }
crate::mutation_payload! { DepositAccountFreezePayload, account: DepositAccount }
crate::mutation_payload! { DepositAccountUnfreezePayload, account: DepositAccount }
crate::mutation_payload! { DepositAccountClosePayload, account: DepositAccount }
