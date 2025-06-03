use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "CustodianConfig",
    err = "CustodianConfigError",
    columns(name(ty = "String", list_by)),
    tbl_prefix = "core"
)]
pub(crate) struct CustodianConfigRepo {
    pool: PgPool,
}

impl CustodianConfigRepo {
    pub(crate) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
