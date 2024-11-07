use sqlx::PgPool;

use super::{error::*, values::*};

#[derive(Clone)]
pub struct CustomerSummaryRepo {
    pool: PgPool,
}

impl CustomerSummaryRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn begin(
        &self,
    ) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, CustomerSummaryError> {
        Ok(self.pool.begin().await?)
    }

    pub async fn persist_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        values: &CustomerSummaryValues,
    ) -> Result<(), CustomerSummaryError> {
        let id = values.id;
        let values = serde_json::to_value(values).expect("Could not serialize customer summary");
        sqlx::query!(
            r#"
            INSERT INTO customer_summaries (id, customer_summary_json)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE
            SET customer_summary_json = $2, modified_at = NOW()
            "#,
            id,
            values
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn load(
        &self,
        id: uuid::Uuid,
    ) -> Result<CustomerSummaryValues, CustomerSummaryError> {
        let row = sqlx::query!(
            r#" 
            SELECT customer_summary_json
            FROM customer_summaries
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        let res = if let Some(row) = row {
            let values: CustomerSummaryValues = serde_json::from_value(row.customer_summary_json)
                .expect("Could not de-serialize customer summary");
            values
        } else {
            CustomerSummaryValues::default()
        };
        Ok(res)
    }
}
