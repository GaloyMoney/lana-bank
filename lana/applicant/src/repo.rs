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

    pub(super) async fn begin_op(&self) -> Result<es_entity::DbOp<'static>, ApplicantError> {
        Ok(es_entity::DbOp::init(&self.pool).await?)
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

    pub async fn find_webhook_data_by_id(&self, id: i64) -> Result<WebhookData, ApplicantError> {
        let row = sqlx::query!(
            r#"
            SELECT customer_id AS "customer_id: CustomerId", content, recorded_at
            FROM sumsub_callbacks
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(WebhookData {
            customer_id: row.customer_id,
            webhook_data: row.content,
            timestamp: row.recorded_at,
        })
    }
}
