use async_graphql::*;

use crate::primitives::*;

pub use lana_app::custody::{Wallet as DomainWallet, WalletNetwork};

#[derive(SimpleObject, Clone)]
#[graphql(name = "Wallet", complex)]
pub struct WalletBase {
    id: ID,
    wallet_id: UUID,

    #[graphql(skip)]
    pub entity: Arc<DomainWallet>,
}

impl From<DomainWallet> for WalletBase {
    fn from(wallet: DomainWallet) -> Self {
        Self {
            id: wallet.id.to_global_id(),
            wallet_id: wallet.id.into(),
            entity: Arc::new(wallet),
        }
    }
}

#[ComplexObject]
impl WalletBase {
    async fn address(&self) -> &str {
        &self.entity.address
    }

    async fn network(&self) -> WalletNetwork {
        self.entity.network
    }

    async fn custodian(&self, ctx: &Context<'_>) -> async_graphql::Result<Custodian> {
        let (app, _sub) = app_and_sub_from_ctx!(ctx);
        let custodians: std::collections::HashMap<_, Custodian> = app
            .custody()
            .find_all_custodians(&[self.entity.custodian_id])
            .await?;
        Ok(custodians
            .into_values()
            .next()
            .expect("wallet must have a custodian"))
    }
}

pub use lana_app::custody::custodian::Custodian as DomainCustodian;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Custodian {
    id: ID,
    custodian_id: UUID,
    created_at: Timestamp,
    #[graphql(skip)]
    pub entity: Arc<DomainCustodian>,
}

impl From<DomainCustodian> for Custodian {
    fn from(custodian: DomainCustodian) -> Self {
        Self {
            id: custodian.id.to_global_id(),
            custodian_id: custodian.id.into(),
            created_at: custodian.created_at().into(),
            entity: Arc::new(custodian),
        }
    }
}

#[ComplexObject]
impl Custodian {
    async fn name(&self) -> &str {
        &self.entity.name
    }
}
