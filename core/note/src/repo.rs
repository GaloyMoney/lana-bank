use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::entity::*;

#[derive(EsRepo)]
#[es_repo(
    entity = "Note",
    columns(
        target_type(ty = "NoteTargetType"),
        target_id(ty = "String", list_for(by(created_at))),
    ),
    tbl_prefix = "core",
    delete = "soft_without_queries"
)]
pub struct NoteRepo {
    pool: PgPool,
}

impl Clone for NoteRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl NoteRepo {
    pub(super) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
