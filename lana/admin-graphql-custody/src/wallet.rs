use async_graphql::*;

use crate::primitives::*;

pub use lana_app::custody::{Wallet as DomainWallet, WalletNetwork};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
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
}
