use sqlx::PgPool;

use es_entity::*;

use crate::primitives::{DepositAccountId, DepositId};

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "Deposit",
    err = "DepositError",
    columns(
        deposit_account_id(ty = "DepositAccountId", list_for, update(persist = false)),
        reference(ty = "String", create(accessor = "reference()"))
    ),
    tbl_prefix = "core"
)]
pub struct DepositRepo {
    pool: PgPool,
}

impl DepositRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
