use sqlx::PgPool;
use tracing_macros::record_error_severity;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "PublicIdEntity",
    event = PublicIdEvent,
    id = "PublicId",
    err = "PublicIdError",
    columns(target_id(ty = "PublicIdTargetId"),),
    tbl = "core_public_ids",
    events_tbl = "core_public_id_events"
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

    #[record_error_severity]
    #[tracing::instrument(name = "public_id.next_counter", skip_all)]
    pub async fn next_counter(&self) -> Result<PublicId, PublicIdError> {
        let result = sqlx::query!("SELECT nextval('core_public_id_counter') as counter")
            .fetch_one(&self.pool)
            .await?;

        let counter = result.counter.unwrap_or(0);
        Ok(PublicId::new(counter.to_string()))
    }
}
