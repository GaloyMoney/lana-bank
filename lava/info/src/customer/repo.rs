use sqlx::PgPool;

use super::{error::*, values::*};

#[derive(Clone)]
pub struct CustomerInfoRepo {
    pool: PgPool,
}

impl CustomerInfoRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn begin(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, CustomerInfoError> {
        Ok(self.pool.begin().await?)
    }

    pub async fn persist_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        values: &CustomerInfoValues,
    ) -> Result<(), CustomerInfoError> {
        let id = values.id;
        let values = serde_json::to_value(values).expect("Could not serialize customer info");
        sqlx::query!(
            r#"
            INSERT INTO customer_infos (id, customer_info_json)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE
            SET customer_info_json = $2, modified_at = NOW()
            "#,
            id,
            values
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn load(&self, id: uuid::Uuid) -> Result<CustomerInfoValues, CustomerInfoError> {
        let row = sqlx::query!(
            r#" 
            SELECT customer_info_json
            FROM customer_infos
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        let res = if let Some(row) = row {
            let values: CustomerInfoValues = serde_json::from_value(row.customer_info_json)
                .expect("Could not de-serialize customer info");
            values
        } else {
            CustomerInfoValues::default()
        };
        Ok(res)
    }
}
