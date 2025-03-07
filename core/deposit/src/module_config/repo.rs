use sqlx::PgPool;

use es_entity::*;

use crate::primitives::DepositConfigId;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "DepositConfig",
    err = "DepositConfigError",
    columns(reference(ty = "String")),
    tbl_prefix = "core"
)]
pub struct DepositConfigRepo {
    pool: PgPool,
}

impl DepositConfigRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
