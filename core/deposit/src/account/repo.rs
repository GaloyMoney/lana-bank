use sqlx::PgPool;

use es_entity::*;

use crate::account::primitives::DepositAccountShortCodeId;
use crate::primitives::{DepositAccountHolderId, DepositAccountId};

use super::{entity::*, error::*};
use sqlx::Transaction;

#[derive(EsRepo, Clone)]
#[es_repo(
    entity = "DepositAccount",
    err = "DepositAccountError",
    columns(
        account_holder_id(ty = "DepositAccountHolderId", list_for, update(persist = false)),
        short_code_id(ty = "DepositAccountShortCodeId")
    ),
    tbl_prefix = "core"
)]
pub struct DepositAccountRepo {
    #[allow(dead_code)]
    pool: PgPool,
}

impl DepositAccountRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn next_short_code_id(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<DepositAccountShortCodeId, DepositAccountError> {
        let short_code_id_val: i64 =
            sqlx::query_scalar!("SELECT nextval('core_deposit_accounts_short_code_id_seq')")
                .fetch_one(&mut **tx)
                .await?
                .ok_or(DepositAccountError::CouldNotGenerateShortCodeId)?;

        short_code_id_val.try_into()
    }
}
