use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;

use super::{AccountingTemplateId, entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "AccountingTemplate",
    err = "AccountingTemplateError",
    columns(name(ty = "String", list_by), code(ty = "String", list_by)),
    tbl_prefix = "core"
)]
pub struct AccountingTemplateRepo {
    pool: PgPool,
    clock: ClockHandle,
}

impl AccountingTemplateRepo {
    pub fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            clock,
        }
    }
}
