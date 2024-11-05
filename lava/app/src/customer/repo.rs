use sqlx::PgPool;

use es_entity::*;

use crate::{data_export::Export, primitives::*};

use super::{entity::*, error::*};

const BQ_TABLE_NAME: &str = "customer_events";

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Customer",
    err = "CustomerError",
    columns(email = "String", telegram_id = "String"),
    post_persist_hook = "export"
)]
pub struct CustomerRepo {
    pool: PgPool,
    export: Export,
}

impl CustomerRepo {
    pub(super) fn new(pool: &PgPool, export: &Export) -> Self {
        Self {
            pool: pool.clone(),
            export: export.clone(),
        }
    }

    async fn export(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        _: &Customer,
        events: impl Iterator<Item = &PersistedEvent<CustomerEvent>>,
    ) -> Result<(), CustomerError> {
        self.export
            .es_entity_export(db, BQ_TABLE_NAME, events)
            .await?;
        Ok(())
    }
}
