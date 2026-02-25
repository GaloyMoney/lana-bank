use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Document",
    err = "DocumentStorageError",
    columns(reference_id(ty = "ReferenceId", list_for(by(created_at)), update(persist = false))),
    tbl_prefix = "core",
    delete = "soft"
)]
pub struct DocumentRepo {
    pool: PgPool,
    clock: ClockHandle,
}

impl Clone for DocumentRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            clock: self.clock.clone(),
        }
    }
}

impl DocumentRepo {
    pub(super) fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            clock,
        }
    }
}
