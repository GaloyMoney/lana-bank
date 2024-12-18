use async_graphql::*;

use crate::primitives::*;

pub use lana_app::deposit::DepositAccount as DomainDepositAccount;

use super::{customer::Customer, deposit::*, withdrawal::*};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct DepositAccount {
    id: ID,
    account_holder_id: UUID,
    ledger_account_id: UUID,
    created_at: Timestamp,
}

impl From<DomainDepositAccount> for DepositAccount {
    fn from(account: DomainDepositAccount) -> Self {
        DepositAccount {
            id: account.id.to_global_id(),
            account_holder_id: account.account_holder_id.into(),
            ledger_account_id: account.ledger_account_id.into(),
            created_at: account.created_at().into(),
        }
    }
}

#[derive(SimpleObject)]
pub struct DepositAccountBalance {
    checking: Checking,
}

impl From<lana_app::deposit::DepositAccountBalance> for DepositAccountBalance {
    fn from(balance: lana_app::deposit::DepositAccountBalance) -> Self {
        Self {
            checking: Checking {
                settled: balance.settled,
                pending: balance.pending,
            },
        }
    }
}

#[derive(SimpleObject)]
struct Checking {
    settled: UsdCents,
    pending: UsdCents,
}

#[ComplexObject]
impl DepositAccount {
    async fn deposits(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Deposit>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let deposits = app
            .deposits()
            .list_deposits_for_account(sub, self.ledger_account_id)
            .await?;
        Ok(deposits.into_iter().map(Deposit::from).collect())
    }

    async fn withdrawals(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Withdrawal>> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let withdrawals = app
            .deposits()
            .list_withdrawals_for_account(sub, self.ledger_account_id)
            .await?;
        Ok(withdrawals.into_iter().map(Withdrawal::from).collect())
    }

    async fn balance(&self, ctx: &Context<'_>) -> async_graphql::Result<DepositAccountBalance> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let balance = app.deposits().balance(sub, self.ledger_account_id).await?;
        Ok(DepositAccountBalance::from(balance))
    }

    async fn customer(&self, ctx: &Context<'_>) -> async_graphql::Result<Customer> {
        let (app, sub) = crate::app_and_sub_from_ctx!(ctx);
        let customer = app
            .customers()
            .find_by_id(sub, self.account_holder_id)
            .await?
            .expect("customer not found");

        Ok(Customer::from(customer))
    }
}
