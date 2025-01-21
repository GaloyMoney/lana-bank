use sqlx::PgPool;

use es_entity::*;

use crate::primitives::StatementId;

use super::entity::*;

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Statement",
    err = "StatementError",
    columns(reference(ty = "String")),
    tbl_prefix = "core"
)]
pub struct StatementRepo {
    pool: PgPool,
}

impl StatementRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
