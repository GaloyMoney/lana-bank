use sqlx::PgPool;
use sqlx::types::uuid;

use core_customer::CustomerId;

#[derive(Clone)]
pub struct CustomerActivityRepo {
    pool: PgPool,
}

impl CustomerActivityRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_activity(
        &self,
        customer_id: CustomerId,
        activity_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), sqlx::Error> {
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

    pub async fn find_customers_in_range_with_non_matching_activity(
        &self,
        start_threshold: chrono::DateTime<chrono::Utc>,
        end_threshold: chrono::DateTime<chrono::Utc>,
        activity: core_customer::Activity,
    ) -> Result<Vec<CustomerId>, sqlx::Error> {
        let activity_str = activity.to_string();
        let rows = sqlx::query!(
            r#"
            SELECT ca.customer_id
            FROM customer_activity ca
            JOIN core_customers c ON ca.customer_id = c.id
            WHERE ca.last_activity_date >= $1 
              AND ca.last_activity_date < $2 
              AND c.activity != $3
            ORDER BY ca.last_activity_date ASC
            "#,
            start_threshold,
            end_threshold,
            activity_str
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| CustomerId::from(row.customer_id))
            .collect())
    }
}
