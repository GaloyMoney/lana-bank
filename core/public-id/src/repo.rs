use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "PublicIdEntity",
    id = "Id",
    err = "PublicIdError",
    columns(target_id(ty = "PublicIdTargetId"),),
    tbl = "core_public_ids",
    events_tbl = "core_public_id_events",
    tbl_prefix = "core"
)]
pub struct PublicIdRepo {
    pool: PgPool,
}

impl Clone for PublicIdRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl PublicIdRepo {
    pub(super) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn next_counter(&self) -> Result<Id, PublicIdError> {
        let result = sqlx::query!("SELECT nextval('core_public_id_counter') as counter")
            .fetch_one(&self.pool)
            .await?;

        let counter = result.counter.unwrap_or(0);
        Ok(Id::new(counter.to_string()))
    }
}
