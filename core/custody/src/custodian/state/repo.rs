use sqlx::{PgPool, Postgres, Transaction, query};
use uuid::Uuid;

use crate::primitives::CustodianId;

use super::error::CustodianStateError;

#[derive(Clone)]
pub struct CustodianStateRepo {
    pool: PgPool,
}

impl CustodianStateRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn persist_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        custodian_id: CustodianId,
        state: serde_json::Value,
    ) -> Result<(), CustodianStateError> {
        let custodian_id: Uuid = custodian_id.into();

        query!(
            r#"
            INSERT INTO core_custodian_states (id, state)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET state = $2
            "#,
            custodian_id,
            state
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn load(
        &self,
        custodian_id: CustodianId,
    ) -> Result<serde_json::Value, CustodianStateError> {
        let custodian_id: Uuid = custodian_id.into();

        let row = query!(
            "SELECT state FROM core_custodian_states WHERE id = $1 ",
            custodian_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.state).unwrap_or(serde_json::Value::Null))
    }
}
