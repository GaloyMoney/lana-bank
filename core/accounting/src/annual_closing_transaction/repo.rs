use sqlx::PgPool;

use es_entity::*;

use crate::primitives::{AnnualClosingTransactionId, CalaTxId};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "AnnualClosingTransaction",
    err = "AnnualClosingTransactionError",
    columns(
        reference(ty = "String", create(accessor = "reference()")),
        ledger_transaction_id(ty = "CalaTxId")
    ),
    tbl_prefix = "core"
)]
pub struct AnnualClosingTransactionRepo {
    pool: PgPool,
}

impl Clone for AnnualClosingTransactionRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl AnnualClosingTransactionRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
