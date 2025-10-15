use sqlx::PgPool;

use es_entity::*;
// TODO: New types? Use ChartId or CalaAccountSetId for an FK
// relationship?
use crate::primitives::{LedgerClosingId};

use super::{entity::*, error::*};
// TODO: Simple FK relationship to core_charts?
#[derive(EsRepo)]
#[es_repo(
    entity = "LedgerClosing",
    err = "LedgerClosingError",
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
