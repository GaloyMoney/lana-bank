use async_graphql::{connection::*, *};

use crate::primitives::*;

pub use admin_graphql_deposit::{
    DepositAccountBase, DepositAccountHistoryCursor, DomainDepositAccount,
};

pub use lana_app::deposit::DepositAccountHistoryEntry as DomainDepositAccountHistoryEntry;

use super::{
    accounting::LedgerAccount, customer::Customer, deposit::*, deposit_account_history::*,
    loader::LanaDataLoader, withdrawal::*,
};

// ===== DepositAccountLedgerAccounts (stays in admin-server, cross-domain DataLoader) =====

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct DepositAccountLedgerAccounts {
    deposit_account_id: UUID,
    frozen_deposit_account_id: UUID,
}

#[ComplexObject]
impl DepositAccountLedgerAccounts {
    async fn deposit_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let account = loader
            .load_one(LedgerAccountId::from(self.deposit_account_id))
            .await?
            .expect("Ledger account not found");
        Ok(account)
    }

    async fn frozen_deposit_account(&self, ctx: &Context<'_>) -> Result<LedgerAccount> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        let account = loader
            .load_one(LedgerAccountId::from(self.frozen_deposit_account_id))
            .await?
            .expect("Ledger account not found");
        Ok(account)
    }
}

// ===== DepositAccount =====

#[derive(Clone)]
pub(super) struct DepositAccountCrossDomain {
    entity: Arc<DomainDepositAccount>,
}

#[Object]
impl DepositAccountCrossDomain {
    async fn deposits(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Deposit>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let deposits = app
            .deposits()
            .list_deposits_for_account(sub, self.entity.id)
            .await?;
        Ok(deposits.into_iter().map(Deposit::from).collect())
    }

    async fn withdrawals(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Withdrawal>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let withdrawals = app
            .deposits()
            .list_withdrawals_for_account(sub, self.entity.id)
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

    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let customer = app
            .customers()
            .find_by_id(sub, self.entity.account_holder_id)
            .await?
            .expect("customer not found");

        Ok(Customer::from(customer))
    }

    async fn ledger_accounts(&self) -> DepositAccountLedgerAccounts {
        DepositAccountLedgerAccounts {
            deposit_account_id: self.entity.account_ids.deposit_account_id.into(),
            frozen_deposit_account_id: self.entity.account_ids.frozen_deposit_account_id.into(),
        }
    }
}

#[derive(MergedObject, Clone)]
#[graphql(name = "DepositAccount")]
pub struct DepositAccount(pub DepositAccountBase, DepositAccountCrossDomain);

impl From<DomainDepositAccount> for DepositAccount {
    fn from(account: DomainDepositAccount) -> Self {
        let base = DepositAccountBase::from(account);
        let cross = DepositAccountCrossDomain {
            entity: base.entity.clone(),
        };
        Self(base, cross)
    }
}

impl std::ops::Deref for DepositAccount {
    type Target = DepositAccountBase;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
