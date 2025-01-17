use sqlx::PgPool;

use es_entity::*;

use crate::primitives::ReportId;

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(entity = "Report", err = "ReportError")]
pub struct ReportRepo {
    pool: PgPool,
}

impl ReportRepo {
    pub(super) fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
