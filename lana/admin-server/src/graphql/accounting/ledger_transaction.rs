use async_graphql::*;

pub use lana_app::{
    accounting::ledger_transaction::{
        LedgerTransaction as DomainLedgerTransaction, LedgerTransactionCursor,
    },
    deposit::{DEPOSIT_TRANSACTION_ENTITY_TYPE, WITHDRAWAL_TRANSACTION_ENTITY_TYPE},
};

use crate::{
    graphql::{deposit::Deposit, loader::*, withdrawal::Withdrawal},
    primitives::*,
};

use super::JournalEntry;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct LedgerTransaction {
    id: ID,
    ledger_transaction_id: UUID,
    created_at: Timestamp,
    effective: Date,
    #[graphql(skip)]
    pub entity: Arc<DomainLedgerTransaction>,
}
#[derive(Union)]
pub enum LedgerTransactionEntity {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
}

#[ComplexObject]
impl LedgerTransaction {
    async fn description(&self) -> &Option<String> {
        &self.entity.description
    }

    async fn entity(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<LedgerTransactionEntity>> {
        let Some(ref entity_ref) = self.entity.entity_ref else {
            return Ok(None);
        };
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let res = match &entity_ref.entity_type {
            entity_type if entity_type == &DEPOSIT_TRANSACTION_ENTITY_TYPE => {
                let deposit = loader
                    .load_one(DepositId::from(entity_ref.entity_id))
                    .await?
                    .expect("Could not find deposit entity");
                Some(LedgerTransactionEntity::Deposit(deposit))
            }
            entity_type if entity_type == &WITHDRAWAL_TRANSACTION_ENTITY_TYPE => {
                let withdrawal = loader
                    .load_one(WithdrawalId::from(entity_ref.entity_id))
                    .await?
                    .expect("Could not find withdrawal entity");
                Some(LedgerTransactionEntity::Withdrawal(withdrawal))
            }
            _ => None,
        };
        Ok(res)
    }

    async fn entries(&self) -> Vec<JournalEntry> {
        self.entity
            .entries
            .iter()
            .map(|e| {
                let entry = e.clone();
                JournalEntry::from(entry)
            })
            .collect()
    }
}

impl From<DomainLedgerTransaction> for LedgerTransaction {
    fn from(tx: DomainLedgerTransaction) -> Self {
        Self {
            id: tx.id.to_global_id(),
            created_at: tx.created_at.into(),
            effective: tx.effective.into(),
            ledger_transaction_id: tx.id.into(),
            entity: Arc::new(tx),
        }
    }
}
