use async_graphql::*;

use crate::{graphql::loader::LanaDataLoader, primitives::*};

pub use lana_app::custody::{Wallet as DomainWallet, WalletNetwork};

use super::Custodian;

#[derive(SimpleObject, Clone)]
#[graphql(
    complex,
    directive = crate::graphql::entity_key::entity_key::apply("walletId".to_string())
)]
pub struct Wallet {
    wallet_id: UUID,

    #[graphql(skip)]
    pub(crate) entity: Arc<DomainWallet>,
}

impl From<DomainWallet> for Wallet {
    fn from(wallet: DomainWallet) -> Self {
        Self {
            wallet_id: wallet.id.into(),
            entity: Arc::new(wallet),
        }
    }
}

#[ComplexObject]
impl Wallet {
    async fn address(&self) -> &str {
        &self.entity.address
    }

    async fn network(&self) -> WalletNetwork {
        self.entity.network
    }

    async fn custodian(&self, ctx: &Context<'_>) -> async_graphql::Result<Custodian> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        loader
            .load_one(self.entity.custodian_id)
            .await?
            .ok_or_else(|| Error::new("Custodian not found"))
    }
}
