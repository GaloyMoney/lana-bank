use async_graphql::{connection::*, *};

use crate::primitives::*;

pub(super) use lana_app::deposit::{
    DepositAccount as DomainDepositAccount, DepositAccountHistoryCursor,
    DepositAccountHistoryEntry as DomainDepositAccountHistoryEntry,
};

use super::{deposit::*, deposit_account_history::*, withdrawal::*};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub(super) struct DepositAccount {
    id: ID,
    deposit_account_id: UUID,
    customer_id: UUID,
    created_at: Timestamp,

    #[graphql(skip)]
    pub(super) entity: Arc<DomainDepositAccount>,
}

impl From<DomainDepositAccount> for DepositAccount {
    fn from(account: DomainDepositAccount) -> Self {
        DepositAccount {
            id: account.id.to_global_id(),
            deposit_account_id: account.id.into(),
            customer_id: account.account_holder_id.into(),
            created_at: account.created_at().into(),

            entity: Arc::new(account),
        }
    }
}

#[derive(SimpleObject)]
pub(super) struct DepositAccountBalance {
    settled: UsdCents,
    pending: UsdCents,
}

impl From<lana_app::deposit::DepositAccountBalance> for DepositAccountBalance {
    fn from(balance: lana_app::deposit::DepositAccountBalance) -> Self {
        Self {
            settled: balance.settled,
            pending: balance.pending,
        }
    }
}

#[ComplexObject]
impl DepositAccount {
    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<DepositAccountBalance> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let balance = app
            .deposits()
            .for_subject(sub)?
            .account_balance(self.entity.id)
            .await?;
        Ok(DepositAccountBalance::from(balance))
    }

    async fn deposits(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Deposit>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let deposits = app
            .deposits()
            .for_subject(sub)?
            .list_deposits_for_account(self.entity.id)
            .await?;
        Ok(deposits.into_iter().map(Deposit::from).collect())
    }

    async fn withdrawals(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Withdrawal>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let withdrawals = app
            .deposits()
            .for_subject(sub)?
            .list_withdrawals_for_account(self.entity.id)
            .await?;
        Ok(withdrawals.into_iter().map(Withdrawal::from).collect())
    }

    async fn history(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
    ) -> async_graphql::Result<
        Connection<
            DepositAccountHistoryCursor,
            DepositAccountHistoryEntry,
            EmptyFields,
            EmptyFields,
        >,
    > {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);

        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let query_args = es_entity::PaginatedQueryArgs { first, after };
                let res = app
                    .deposits()
                    .for_subject(sub)?
                    .account_history(self.entity.id, query_args)
                    .await?;

                let mut connection = Connection::new(false, res.has_next_page);
                connection.edges.extend(
                    res.entities
                        .into_iter()
                        .filter(|entry| !matches!(entry, DomainDepositAccountHistoryEntry::Ignored))
                        .map(|entry| {
                            let cursor = DepositAccountHistoryCursor::from(&entry);
                            Edge::new(cursor, DepositAccountHistoryEntry::from(entry))
                        }),
                );
                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }
}
