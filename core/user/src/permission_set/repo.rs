use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "PermissionSet",
    err = "PermissionSetError",
    tbl_prefix = "core"
)]
pub(crate) struct PermissionSetRepo {
    pool: PgPool,
}

impl PermissionSetRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
