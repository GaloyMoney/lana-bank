use sqlx::PgPool;

use crate::primitives::{ChartId, LedgerClosingId};
use es_entity::*;

use super::{entity::*, error::*};
#[derive(EsRepo)]
#[es_repo(
    entity = "LedgerClosing",
    err = "LedgerClosingError",
    columns(chart_id(ty = "ChartId")),
    tbl_prefix = "core"
)]
pub struct LedgerClosingRepo {
    pool: PgPool,
}

impl Clone for LedgerClosingRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl LedgerClosingRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
