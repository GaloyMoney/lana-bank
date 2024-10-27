use sqlx::PgPool;

use es_entity::*;

use crate::{
    data_export::Export,
    primitives::{CustomerId, WithdrawId},
};

use super::{entity::*, error::*};

const BQ_TABLE_NAME: &str = "withdraw_events";

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Withdraw",
    err = "WithdrawError",
    columns(
        customer_id(ty = "CustomerId", list_for),
        reference(ty = "String", create(accessor = "reference()")),
    ),
    post_persist_hook = "export"
)]
pub struct WithdrawRepo {
    pool: PgPool,
    export: Export,
}

impl WithdrawRepo {
    pub(super) fn new(pool: &PgPool, export: &Export) -> Self {
        Self {
            pool: pool.clone(),
            export: export.clone(),
        }
    }

    async fn export(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        events: impl Iterator<Item = &PersistedEvent<WithdrawEvent>>,
    ) -> Result<(), WithdrawError> {
        self.export
            .es_entity_export(db, BQ_TABLE_NAME, events)
            .await?;
        Ok(())
    }
}
