use sqlx::PgPool;

use es_entity::*;

use crate::primitives::ChartOfAccountId;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "ChartOfAccount",
    err = "ChartOfAccountError",
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
