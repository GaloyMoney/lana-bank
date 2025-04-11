use sqlx::PgPool;

use es_entity::*;

use crate::primitives::{AccountingCsvId, LedgerAccountId};

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "AccountingCsv",
    err = "AccountingCsvError",
    columns(csv_type(ty = "AccountingCsvType", list_by = true))
)]
pub struct AccountingCsvRepo {
    pool: PgPool,
}

impl AccountingCsvRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
