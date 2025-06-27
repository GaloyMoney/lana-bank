use serde::{Serialize, de::DeserializeOwned};
use sqlx::{PgPool, query};
use uuid::Uuid;

use super::error::CustodianStateError;

#[derive(Debug, Clone)]
pub struct CustodianStateRepo {
    pool: PgPool,
}

impl CustodianStateRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn load<T: DeserializeOwned + Default>(
        &self,
        custodian_id: Uuid,
    ) -> Result<T, CustodianStateError> {
        let row = query!(
            "SELECT state FROM core_custodian_states WHERE id = $1 ",
            custodian_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row
            .map(|r| serde_json::from_value(r.state))
            .transpose()?
            .unwrap_or_default())
    }

    pub async fn persist<T: Serialize>(
        &self,
        custodian_id: Uuid,
        state: &T,
    ) -> Result<(), CustodianStateError> {
        query!(
            r#"
            INSERT INTO core_custodian_states (id, state)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET state = $2
            "#,
            custodian_id,
            serde_json::to_value(state).expect("successful encoding")
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
