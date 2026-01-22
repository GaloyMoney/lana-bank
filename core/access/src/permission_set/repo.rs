use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "PermissionSet",
    err = "PermissionSetError",
    columns(name(ty = "String", list_by)),
    tbl_prefix = "core"
)]
pub(crate) struct PermissionSetRepo {
    #[allow(dead_code)]
    pool: PgPool,
    #[allow(dead_code)]
    clock: ClockHandle,
}

impl PermissionSetRepo {
    pub fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            clock,
        }
    }
}
