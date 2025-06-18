use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Document",
    err = "DocumentStorageError",
    columns(reference_id(ty = "Option<ReferenceId>", list_for, update(persist = false))),
    tbl_prefix = "core",
    delete = "soft"
)]
pub struct DocumentRepo {
    pool: PgPool,
}

impl Clone for DocumentRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl DocumentRepo {
    pub(super) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
