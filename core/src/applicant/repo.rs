use crate::primitives::CustomerId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use sqlx::{PgPool, Postgres, Transaction};

use super::error::ApplicantError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApplicantEvent {
    WebhookReceived {
        customer_id: CustomerId,
        webhook_data: serde_json::Value,
        #[serde(with = "chrono::serde::ts_milliseconds")]
        timestamp: chrono::DateTime<Utc>,
    },
}

#[derive(Clone)]
pub struct ApplicantRepo {
    pool: PgPool,
}

impl ApplicantRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn persist_webhook(
        &self,
        db: &mut Transaction<'_, Postgres>,
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
        .fetch_one(&mut **db)
        .await?;

        Ok(row.id)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<ApplicantEvent, ApplicantError> {
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

        Ok(ApplicantEvent::WebhookReceived {
            customer_id: row.customer_id,
            webhook_data: row.content,
            timestamp: row.recorded_at,
        })
    }
}
