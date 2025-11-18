use sqlx::PgPool;
use sqlx::types::uuid;
use tracing::{Span, field};

use crate::{Activity, CustomerError, CustomerId};

#[derive(Clone)]
pub struct CustomerActivityRepo {
    pool: PgPool,
}

impl CustomerActivityRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[tracing::instrument(
        name = "customer_activity.upsert_activity",
        skip(self),
        fields(customer_id = %customer_id),
        err(level = "warn")
    )]
    pub async fn upsert_activity(
        &self,
        customer_id: CustomerId,
        activity_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), CustomerError> {
        let customer_uuid: uuid::Uuid = customer_id.into();
        sqlx::query!(
            r#"
            INSERT INTO customer_activity (customer_id, last_activity_date)
            VALUES ($1, $2)
            ON CONFLICT (customer_id) 
            DO UPDATE SET 
                last_activity_date = GREATEST(COALESCE(customer_activity.last_activity_date, $2), $2)

            "#,
            customer_uuid,
            activity_date,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(
        name = "customer_activity.find_customers_needing_activity_update",
        skip(self),
        fields(matched_count = field::Empty),
        err(level = "warn")
    )]
    pub async fn find_customers_needing_activity_update(
        &self,
        start_threshold: chrono::DateTime<chrono::Utc>,
        end_threshold: chrono::DateTime<chrono::Utc>,
        activity: Activity,
    ) -> Result<Vec<CustomerId>, CustomerError> {
        let activity_str = activity.to_string();
        let rows = sqlx::query!(
            r#"
            SELECT ca.customer_id
            FROM customer_activity ca
            JOIN core_customers c ON ca.customer_id = c.id
            WHERE ca.last_activity_date >= $1 
              AND ca.last_activity_date < $2 
              AND c.activity != $3
            "#,
            start_threshold,
            end_threshold,
            activity_str
        )
        .fetch_all(&self.pool)
        .await?;

        let customer_ids: Vec<CustomerId> = rows
            .into_iter()
            .map(|row| CustomerId::from(row.customer_id))
            .collect();

        Span::current().record("matched_count", customer_ids.len());

        Ok(customer_ids)
    }
}
