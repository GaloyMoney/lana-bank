use async_graphql::dataloader::{DataLoader, Loader};
use tracing::instrument;

use std::collections::HashMap;

use lana_app::{
    app::LanaApp,
    customer::{Party, PartyId},
};

use crate::{Customer, Prospect, primitives::*};

pub type LanaDataLoader = DataLoader<LanaLoader>;
pub struct LanaLoader {
    pub app: LanaApp,
}

impl LanaLoader {
    pub fn new(app: &LanaApp) -> LanaDataLoader {
        DataLoader::new(
            Self { app: app.clone() },
            async_graphql::runtime::TokioSpawner::current(),
            async_graphql::runtime::TokioTimer::default(),
        )
        // Set delay to 0 as per https://github.com/async-graphql/async-graphql/issues/1306
        .delay(std::time::Duration::from_millis(5))
    }
}

impl Loader<CustomerId> for LanaLoader {
    type Value = Customer;
    type Error = Arc<lana_app::customer::error::CustomerError>;

    #[instrument(name = "loader.customers", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[CustomerId],
    ) -> Result<HashMap<CustomerId, Customer>, Self::Error> {
        self.app.customers().find_all(keys).await.map_err(Arc::new)
    }
}

impl Loader<PartyId> for LanaLoader {
    type Value = Arc<Party>;
    type Error = Arc<lana_app::customer::error::CustomerError>;

    #[instrument(name = "loader.parties", skip(self), fields(count = keys.len()), err)]
    async fn load(&self, keys: &[PartyId]) -> Result<HashMap<PartyId, Arc<Party>>, Self::Error> {
        self.app
            .customers()
            .find_all_parties(keys)
            .await
            .map_err(Arc::new)
    }
}

impl Loader<ProspectId> for LanaLoader {
    type Value = Prospect;
    type Error = Arc<lana_app::customer::error::CustomerError>;

    #[instrument(name = "loader.prospects", skip(self), fields(count = keys.len()), err)]
    async fn load(
        &self,
        keys: &[ProspectId],
    ) -> Result<HashMap<ProspectId, Prospect>, Self::Error> {
        self.app
            .customers()
            .find_all_prospects(keys)
            .await
            .map_err(Arc::new)
    }
}
