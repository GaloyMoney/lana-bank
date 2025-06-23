use sqlx::PgPool;

use es_entity::*;

use crate::primitives::CommitteeId;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Committee",
    err = "CommitteeError",
    columns(name = "String"),
    tbl_prefix = "core"
)]
pub struct CommitteeRepo {
    #[allow(dead_code)]
    pool: PgPool,
}

impl CommitteeRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
