use async_graphql::*;

use crate::primitives::*;

pub use lana_app::custody::Wallet as DomainWallet;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Wallet {
    id: ID,
    wallet_id: UUID,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainWallet>,
}

impl From<DomainWallet> for Wallet {
    fn from(wallet: DomainWallet) -> Self {
        Self {
            id: wallet.id.to_global_id(),
            wallet_id: wallet.id.into(),
            entity: Arc::new(wallet),
        }
    }
}

#[ComplexObject]
impl Wallet {
    async fn address(&self) -> Option<&str> {
        self.entity.address()
    }
}
