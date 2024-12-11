use sqlx::PgPool;

use es_entity::*;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "ChartOfAccounts",
    err = "ChartOfAccountsError",
    tbl_prefix = "core"
)]
pub struct ChartOfAccountsRepo {
    pool: PgPool,
}

impl ChartOfAccountsRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
