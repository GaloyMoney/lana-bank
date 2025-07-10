use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "PublicRef",
    id = "Ref",
    err = "PublicRefError",
    columns(target_id(ty = "RefTargetId"),),
    tbl_prefix = "core"
)]
pub struct PublicRefRepo {
    pool: PgPool,
}

impl Clone for PublicRefRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl PublicRefRepo {
    pub(super) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn next_counter(&self) -> Result<i64, PublicRefError> {
        let result = sqlx::query!("SELECT nextval('core_public_ref_counter') as counter")
            .fetch_one(&self.pool)
            .await?;

        Ok(result.counter.unwrap_or(0))
    }
}
