use core_customer::CustomerId;

use sqlx::PgPool;

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

    #[tracing::instrument(name = "applicant.persist_webhook_data", skip(self), err)]
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
