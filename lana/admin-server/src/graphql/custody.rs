use async_graphql::*;

use crate::primitives::*;

use super::loader::LanaDataLoader;

pub use admin_graphql_custody::{Custodian, DomainWallet, WalletBase};

// ===== Wallet =====

#[derive(Clone)]
pub(super) struct WalletCrossDomain {
    entity: Arc<DomainWallet>,
}

#[Object]
impl WalletCrossDomain {
    async fn custodian(&self, ctx: &Context<'_>) -> async_graphql::Result<Custodian> {
        let loader = ctx.data_unchecked::<LanaDataLoader>();
        Ok(loader
            .load_one(self.entity.custodian_id)
            .await?
            .expect("wallet must have a custodian"))
    }
}

#[derive(MergedObject, Clone)]
#[graphql(name = "Wallet")]
pub struct Wallet(pub WalletBase, WalletCrossDomain);

impl From<DomainWallet> for Wallet {
    fn from(wallet: DomainWallet) -> Self {
        let base = WalletBase::from(wallet);
        let cross = WalletCrossDomain {
            entity: base.entity.clone(),
        };
        Self(base, cross)
    }
}

impl std::ops::Deref for Wallet {
    type Target = WalletBase;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
