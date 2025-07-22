use chrono::Utc;
use core_customer::CustomerId;
use serde::{Deserialize, Serialize};

use sqlx::PgPool;

use super::error::ApplicantError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookData {
    pub(super) customer_id: CustomerId,
    pub(super) webhook_data: serde_json::Value,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub(super) timestamp: chrono::DateTime<Utc>,
}

#[derive(Clone)]
pub struct ApplicantRepo {
    pool: PgPool,
}

impl ApplicantRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn persist_webhook_data(
        &self,
        customer_id: CustomerId,
        webhook_data: serde_json::Value,
    ) -> Result<i64, ApplicantError> {
        let row = sqlx::query!(
            r#"
            INSERT INTO sumsub_callbacks (customer_id, content)
            VALUES ($1, $2)
            RETURNING id
            "#,
            customer_id as CustomerId,
            webhook_data
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id)
    }
}
