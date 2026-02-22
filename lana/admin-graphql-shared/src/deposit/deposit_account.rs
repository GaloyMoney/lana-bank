use async_graphql::{connection::*, *};

use super::primitives::*;
use super::{
    deposit::DepositBase, deposit_account_history::*, ledger_accounts::*,
    withdrawal::WithdrawalBase,
};

pub use lana_app::deposit::{
    DepositAccount as DomainDepositAccount, DepositAccountHistoryCursor,
    DepositAccountHistoryEntry as DomainDepositAccountHistoryEntry, DepositAccountStatus,
};

#[derive(SimpleObject, Clone)]
#[graphql(name = "DepositAccount", complex)]
pub struct DepositAccountBase {
    id: ID,
    deposit_account_id: UUID,
    customer_id: UUID,
    created_at: Timestamp,
    status: DepositAccountStatus,

    #[graphql(skip)]
    pub entity: Arc<DomainDepositAccount>,
}

impl From<DomainDepositAccount> for DepositAccountBase {
    fn from(account: DomainDepositAccount) -> Self {
        Self {
            id: account.id.to_global_id(),
            deposit_account_id: account.id.into(),
            customer_id: account.account_holder_id.into(),
            created_at: account.created_at().into(),
            status: account.status,

            entity: Arc::new(account),
        }
    }
}

#[ComplexObject]
impl DepositAccountBase {
    async fn public_id(&self) -> &PublicId {
        &self.entity.public_id
    }

    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<DepositAccountBalance> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let balance = app.deposits().account_balance(sub, self.entity.id).await?;
        Ok(DepositAccountBalance::from(balance))
    }

    async fn deposits(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<DepositBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let deposits = app
            .deposits()
            .list_deposits_for_account(sub, self.entity.id)
            .await?;
        Ok(deposits.into_iter().map(DepositBase::from).collect())
    }

    async fn withdrawals(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<WithdrawalBase>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let withdrawals = app
            .deposits()
            .list_withdrawals_for_account(sub, self.entity.id)
            .await?;
        Ok(withdrawals.into_iter().map(WithdrawalBase::from).collect())
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
        let (app, sub) = app_and_sub_from_ctx!(ctx);

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
                    .account_history(sub, self.entity.id, query_args)
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

    async fn ledger_accounts(&self) -> DepositAccountLedgerAccounts {
        DepositAccountLedgerAccounts {
            deposit_account_id: self.entity.account_ids.deposit_account_id.into(),
            frozen_deposit_account_id: self.entity.account_ids.frozen_deposit_account_id.into(),
        }
    }
}

pub use lana_app::public_id::PublicId;

#[derive(SimpleObject)]
pub struct DepositAccountBalance {
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
