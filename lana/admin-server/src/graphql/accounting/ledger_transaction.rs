use async_graphql::*;

pub use lana_app::{
    accounting::ledger_transaction::{
        LedgerTransaction as DomainLedgerTransaction, LedgerTransactionCursor,
        LedgerTransactionInitiator as DomainLedgerTransactionInitiator,
    },
    credit::DISBURSAL_TRANSACTION_ENTITY_TYPE,
    deposit::{DEPOSIT_TRANSACTION_ENTITY_TYPE, WITHDRAWAL_TRANSACTION_ENTITY_TYPE},
};

use crate::{
    graphql::{
        access::User, audit::System, credit_facility::CreditFacilityDisbursal, deposit::Deposit,
        loader::*, withdrawal::Withdrawal,
    },
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
    Disbursal(CreditFacilityDisbursal),
}

#[derive(Union)]
pub enum LedgerTransactionInitiator {
    User(User),
    System(System),
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
            entity_type if entity_type == &DISBURSAL_TRANSACTION_ENTITY_TYPE => {
                let disbursal = loader
                    .load_one(DisbursalId::from(entity_ref.entity_id))
                    .await?
                    .expect("Could not find disbursal entity");
                Some(LedgerTransactionEntity::Disbursal(disbursal))
            }
            _ => None,
        };
        Ok(res)
    }

    async fn initiated_by(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<LedgerTransactionInitiator> {
        match &self.entity.initiated_by {
            DomainLedgerTransactionInitiator::User(id) => {
                let loader = ctx.data_unchecked::<LanaDataLoader>();
                match loader.load_one(UserId::from(*id)).await? {
                    Some(user) => Ok(LedgerTransactionInitiator::User(user)),
                    None => Err("Initiator user not found".into()),
                }
            }
            DomainLedgerTransactionInitiator::System(actor) => Ok(
                LedgerTransactionInitiator::System(System::from_actor(actor.clone())),
            ),
            DomainLedgerTransactionInitiator::Customer(..) => {
                Err("Customer-initiated transactions not supported yet".into())
            }
        }
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
