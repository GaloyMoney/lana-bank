use async_graphql::*;

use std::sync::Arc;

use admin_graphql_shared::primitives::*;

pub use lana_app::custody::{Wallet as DomainWallet, WalletNetwork};

use super::custodian::Custodian;

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
