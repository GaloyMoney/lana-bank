use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(entity = "Role", err = "RoleError", tbl_prefix = "core")]
pub(crate) struct RoleRepo {
    pool: PgPool,
}

impl RoleRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
