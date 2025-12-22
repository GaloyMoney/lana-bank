use core_customer::CustomerId;

use sqlx::PgPool;
use tracing_macros::record_error_severity;

use super::error::ApplicantError;

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

    #[record_error_severity]
    #[tracing::instrument(name = "applicant.persist_webhook_data", skip(self))]
    pub async fn persist_webhook_data(
        &self,
        customer_id: CustomerId,
        webhook_data: serde_json::Value,
    ) -> Result<i64, ApplicantError> {
        let mut op = self.pool.begin().await?;
        self.persist_webhook_data_in_op(&mut op, customer_id, webhook_data)
            .await
    }

    #[record_error_severity]
    #[tracing::instrument(name = "applicant.persist_webhook_data_in_op", skip(self, op))]
    pub(super) async fn persist_webhook_data_in_op(
        &self,
        op: &mut impl es_entity::AtomicOperation,
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
        .fetch_one(op.as_executor())
        .await?;

        Ok(row.id)
    }
}
