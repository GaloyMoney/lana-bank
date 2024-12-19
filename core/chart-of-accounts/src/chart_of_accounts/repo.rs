use sqlx::PgPool;

use es_entity::*;

use crate::primitives::ChartId as ChartOfAccountId;

use super::entity::*;

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "ChartOfAccount",
    err = "ChartOfAccountError",
    columns(reference(ty = "String")),
    tbl_prefix = "core"
)]
pub struct ChartOfAccountRepo {
    pool: PgPool,
}

impl ChartOfAccountRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
