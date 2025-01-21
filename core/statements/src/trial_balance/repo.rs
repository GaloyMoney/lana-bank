use sqlx::PgPool;

use es_entity::*;

use crate::primitives::TrialBalanceStatementId;

use super::entity::*;

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "TrialBalanceStatement",
    err = "TrialBalanceStatementError",
    columns(reference(ty = "String")),
    tbl_prefix = "core"
)]
pub struct TrialBalanceStatementRepo {
    pool: PgPool,
}

impl TrialBalanceStatementRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
