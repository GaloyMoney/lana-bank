use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "Report",
    err = "ReportError",
    columns(external_id(ty = "String"), run_id(ty = "ReportRunId", list_for)),
    tbl_prefix = "core"
)]
pub(crate) struct ReportRepo {
    #[allow(dead_code)]
    pool: PgPool,
}

impl ReportRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}

impl Clone for ReportRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
