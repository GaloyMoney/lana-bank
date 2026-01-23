use es_entity::clock::ClockHandle;
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
    clock: ClockHandle,
}

impl CommitteeRepo {
    pub fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            clock,
        }
    }
}
