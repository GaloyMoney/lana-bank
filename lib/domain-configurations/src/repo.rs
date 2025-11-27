use es_entity::*;
use sqlx::PgPool;
use tracing::instrument;

use crate::{error::*, entity::*, primitives::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "DomainConfiguration",
    id = "DomainConfigurationKey",
    err = "DomainConfigurationError",
    tbl = "domain_configurations",
    events_tbl = "domain_configuration_events"
)]
pub struct DomainConfigurationRepo {
    pool: PgPool,
}

impl Clone for DomainConfigurationRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl DomainConfigurationRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    #[instrument(name = "domain_configuration_repo.persist", skip(self, events), err)]
    pub async fn persist(
        &self,
        key: DomainConfigurationKey,
        events: EntityEvents<DomainConfigurationEvent>,
    ) -> Result<(), DomainConfigurationError> {
        self.persist_events(key, events).await?;
        Ok(())
    }
}
