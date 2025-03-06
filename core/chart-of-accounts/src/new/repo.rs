use sqlx::PgPool;

use es_entity::*;

use crate::{chart_of_accounts::error::ChartError, primitives::ChartId};

use super::entity::*;

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Chart",
    err = "ChartError",
    columns(reference(ty = "String")),
    tbl_prefix = "core_alt"
)]
pub struct ChartRepo {
    pool: PgPool,
}

impl ChartRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
