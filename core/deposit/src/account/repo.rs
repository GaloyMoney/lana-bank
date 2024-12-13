use sqlx::PgPool;

use es_entity::*;

use crate::primitives::{DepositAccountHolderId, DepositAccountId};

use super::{entity::*, error::*};

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "DepositAccount",
    err = "DepositAccountError",
    columns(account_holder_id(ty = "DepositAccountHolderId", list_for, update(persist = false)))
)]
pub struct DepositAccountRepo {
    #[allow(dead_code)]
    pool: PgPool,
}

impl DepositAccountRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }
}
