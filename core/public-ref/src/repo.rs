use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "PublicRef",
    err = "PublicRefError",
    columns(reference(ty = "Ref"),),
    tbl_prefix = "core"
)]
pub struct PublicRefRepo {
    pool: PgPool,
}

impl Clone for PublicRefRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl PublicRefRepo {
    pub(super) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
