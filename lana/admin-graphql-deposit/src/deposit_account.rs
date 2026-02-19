use async_graphql::*;

use crate::primitives::*;

pub use lana_app::deposit::{
    DepositAccount as DomainDepositAccount, DepositAccountHistoryCursor, DepositAccountStatus,
};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
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
}

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
