use sqlx::PgPool;

use es_entity::*;

use crate::{data_export::Export, primitives::*};

use super::{entity::*, error::*};

const BQ_TABLE_NAME: &str = "user_events";

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "User",
    err = "UserError",
    columns(email = "String"),
    post_persist_hook = "export"
)]
pub struct UserRepo {
    pool: PgPool,
    export: Export,
}

impl UserRepo {
    pub(super) fn new(pool: &PgPool, export: &Export) -> Self {
        Self {
            pool: pool.clone(),
            export: export.clone(),
        }
    }

    async fn export(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        events: impl Iterator<Item = &PersistedEvent<UserEvent>>,
    ) -> Result<(), UserError> {
        self.export
            .es_entity_export(db, BQ_TABLE_NAME, events)
            .await?;
        Ok(())
    }
}
