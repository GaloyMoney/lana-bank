use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;

use super::{TermsTemplateId, entity::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "TermsTemplate",
    columns(name(ty = "String", list_by)),
    tbl_prefix = "core"
)]
pub struct TermsTemplateRepo {
    pool: PgPool,
    clock: ClockHandle,
}

impl TermsTemplateRepo {
    pub fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            clock,
        }
    }
}
