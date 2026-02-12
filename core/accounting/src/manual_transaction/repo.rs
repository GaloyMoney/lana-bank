use es_entity::clock::ClockHandle;
use sqlx::PgPool;

use es_entity::*;

use crate::primitives::{CalaTxId, ManualTransactionId};

use super::{entity::*, error::*};

#[derive(EsRepo)]
#[es_repo(
    entity = "ManualTransaction",
    err = "ManualTransactionError",
    columns(
        reference(ty = "String", create(accessor = "reference()")),
        ledger_transaction_id(ty = "CalaTxId")
    ),
    tbl_prefix = "core"
)]
pub(super) struct ManualTransactionRepo {
    pool: PgPool,
    clock: ClockHandle,
}

impl Clone for ManualTransactionRepo {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            clock: self.clock.clone(),
        }
    }
}

impl ManualTransactionRepo {
    pub(super) fn new(pool: &PgPool, clock: ClockHandle) -> Self {
        Self {
            pool: pool.clone(),
            clock,
        }
    }
}
