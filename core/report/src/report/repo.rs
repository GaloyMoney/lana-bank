use sqlx::PgPool;

use es_entity::*;

use crate::primitives::*;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Report",
    err = "ReportError",
    columns(
        external_id(ty = "String"),
        run_id(ty = "ReportRunId", list_for(by(created_at)))
    ),
    tbl_prefix = "core"
)]
pub struct ReportRepo {
    #[allow(dead_code)]
    pool: PgPool,
}

impl ReportRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
