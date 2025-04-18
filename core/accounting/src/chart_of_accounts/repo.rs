use sqlx::PgPool;

use es_entity::*;

use crate::primitives::ChartId;

use super::{entity::*, error::ChartOfAccountsError};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Chart",
    err = "ChartOfAccountsError",
    columns(reference(ty = "String")),
    tbl_prefix = "core"
)]
pub struct ChartRepo {
    pool: PgPool,
}

impl ChartRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
