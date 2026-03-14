use async_graphql::{connection::*, *};
use es_entity::Sort;

use crate::{graphql::accounting::LedgerTransaction, primitives::*};

use super::{
    event_timeline::{self, EventTimelineCursor, EventTimelineEntry},
    loader::LanaDataLoader,
    primitives::SortDirection,
};

pub use super::deposit_account::{DepositAccount, DepositAccountsFilter, DepositAccountsSort};

pub use lana_app::{
    deposit::{
        Deposit as DomainDeposit, DepositAccountsCursor,
        DepositAccountsFilters as DomainDepositAccountsFilters,
        DepositAccountsSortBy as DomainDepositAccountsSortBy, DepositStatus, DepositsCursor,
        DepositsFilters as DomainDepositsFilters, DepositsSortBy as DomainDepositsSortBy,
    },
    public_id::PublicId,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Deposit {
    id: ID,
    deposit_id: UUID,
    account_id: UUID,
    amount: UsdCents,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainDeposit>,
}

impl From<DomainDeposit> for Deposit {
    fn from(deposit: DomainDeposit) -> Self {
        Deposit {
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
impl Deposit {
    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }

    async fn reference(&self) -> &str {
        &self.entity.reference
    }

    async fn status(&self) -> DepositStatus {
        self.entity.status()
    }

    async fn account(&self, ctx: &Context<'_>) -> async_graphql::Result<DepositAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let account = loader
            .load_one(self.entity.deposit_account_id)
            .await?
            .expect("process not found");
        Ok(account)
    }

    async fn event_history(
        &self,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<EventTimelineCursor, EventTimelineEntry, EmptyFields, EmptyFields>,
    > {
        use es_entity::EsEntity as _;
        event_timeline::events_to_connection(self.entity.events(), first, after)
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

#[derive(InputObject)]
pub struct DepositRecordInput {
    pub deposit_account_id: UUID,
    pub amount: UsdCents,
    pub reference: Option<String>,
}
crate::mutation_payload! { DepositRecordPayload, deposit: Deposit }

#[derive(InputObject)]
pub struct DepositAccountCreateInput {
    pub customer_id: UUID,
    #[graphql(default_with = "String::from(\"USD\")")]
    pub currency: String,
}
crate::mutation_payload! { DepositAccountCreatePayload, account: DepositAccount }

#[derive(InputObject)]
pub struct DepositRevertInput {
    pub deposit_id: UUID,
}
crate::mutation_payload! { DepositRevertPayload, deposit: Deposit }

#[derive(InputObject)]
pub struct DepositAccountFreezeInput {
    pub deposit_account_id: UUID,
}
crate::mutation_payload! { DepositAccountFreezePayload, account: DepositAccount }

#[derive(InputObject)]
pub struct DepositAccountUnfreezeInput {
    pub deposit_account_id: UUID,
}
crate::mutation_payload! { DepositAccountUnfreezePayload, account: DepositAccount }

#[derive(InputObject)]
pub struct DepositAccountCloseInput {
    pub deposit_account_id: UUID,
}
crate::mutation_payload! { DepositAccountClosePayload, account: DepositAccount }

#[derive(InputObject)]
pub struct DepositsFilter {
    pub status: Option<DepositStatus>,
}

#[derive(async_graphql::Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DepositsSortBy {
    #[default]
    CreatedAt,
    Amount,
    PublicId,
}

impl From<DepositsSortBy> for DomainDepositsSortBy {
    fn from(by: DepositsSortBy) -> Self {
        match by {
            DepositsSortBy::CreatedAt => DomainDepositsSortBy::CreatedAt,
            DepositsSortBy::Amount => DomainDepositsSortBy::Amount,
            DepositsSortBy::PublicId => DomainDepositsSortBy::PublicId,
        }
    }
}

#[derive(InputObject, Default, Debug, Clone, Copy)]
pub struct DepositsSort {
    #[graphql(default)]
    pub by: DepositsSortBy,
    #[graphql(default)]
    pub direction: SortDirection,
}

impl From<DepositsSort> for Sort<DomainDepositsSortBy> {
    fn from(sort: DepositsSort) -> Self {
        Self {
            by: sort.by.into(),
            direction: sort.direction.into(),
        }
    }
}

impl From<DepositsSort> for DomainDepositsSortBy {
    fn from(sort: DepositsSort) -> Self {
        sort.by.into()
    }
}
