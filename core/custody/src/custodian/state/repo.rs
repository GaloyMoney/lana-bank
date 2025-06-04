use serde::{Serialize, de::DeserializeOwned};
use sqlx::{PgPool, query};
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

    pub fn persisted_state_for(&self, custodian_id: CustodianId) -> PersistedCustodianState<'_> {
        PersistedCustodianState {
            custodian_id,
            pool: &self.pool,
        }
    }
}

pub struct PersistedCustodianState<'a> {
    custodian_id: CustodianId,
    pool: &'a PgPool,
}

impl PersistedCustodianState<'_> {
    pub async fn load<T: DeserializeOwned + Default>(&self) -> Result<T, CustodianStateError> {
        let custodian_id: Uuid = self.custodian_id.into();

        let row = query!(
            "SELECT state FROM core_custodian_states WHERE id = $1 ",
            custodian_id
        )
        .fetch_optional(self.pool)
        .await?;

        Ok(row
            .map(|r| serde_json::from_value(r.state))
            .transpose()?
            .unwrap_or_default())
    }

    pub async fn persist<T: Serialize>(&self, state: &T) -> Result<(), CustodianStateError> {
        let custodian_id: Uuid = self.custodian_id.into();

        query!(
            r#"
            INSERT INTO core_custodian_states (id, state)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET state = $2
            "#,
            custodian_id,
            serde_json::to_value(state).expect("successful encoding")
        )
        .execute(self.pool)
        .await?;

        Ok(())
    }
}
