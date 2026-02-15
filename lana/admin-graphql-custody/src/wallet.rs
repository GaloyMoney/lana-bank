use async_graphql::*;

use crate::primitives::*;

pub use lana_app::custody::{Wallet as DomainWallet, WalletNetwork};

use super::custodian::Custodian;

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Wallet {
    id: ID,
    wallet_id: UUID,

    #[graphql(skip)]
    pub entity: Arc<DomainWallet>,
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
    async fn address(&self) -> &str {
        &self.entity.address
    }

    async fn network(&self) -> WalletNetwork {
        self.entity.network
    }

    async fn custodian(&self, ctx: &Context<'_>) -> async_graphql::Result<Custodian> {
        let app = ctx.data_unchecked::<lana_app::app::LanaApp>();
        let custodians: std::collections::HashMap<CustodianId, Custodian> = app
            .custody()
            .find_all_custodians(&[self.entity.custodian_id])
            .await?;
        Ok(custodians
            .into_values()
            .next()
            .expect("wallet must have a custodian"))
    }
}
