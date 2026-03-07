use sqlx::PgPool;
use sqlx::types::uuid;
use tracing::{Span, field};

use tracing_macros::record_error_severity;

use crate::{DepositAccountActivity, DepositAccountId, error::CoreDepositError};

#[derive(Clone)]
pub struct DepositAccountActivityRepo {
    pool: PgPool,
}

impl DepositAccountActivityRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[record_error_severity]
    #[tracing::instrument(
        name = "deposit_account_activity.upsert_activity",
        skip(self),
        fields(deposit_account_id = %deposit_account_id),
    )]
    pub async fn upsert_activity(
        &self,
        deposit_account_id: DepositAccountId,
        activity_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), CoreDepositError> {
        let account_uuid: uuid::Uuid = deposit_account_id.into();
        sqlx::query!(
            r#"
            INSERT INTO deposit_account_activity (deposit_account_id, last_activity_date)
            VALUES ($1, $2)
            ON CONFLICT (deposit_account_id)
            DO UPDATE SET
                last_activity_date = GREATEST(COALESCE(deposit_account_activity.last_activity_date, $2), $2)

            "#,
            account_uuid,
            activity_date,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[record_error_severity]
    #[tracing::instrument(
        name = "deposit_account_activity.find_accounts_needing_activity_update",
        skip(self),
        fields(matched_count = field::Empty),
    )]
    pub async fn find_accounts_needing_activity_update(
        &self,
        start_threshold: chrono::DateTime<chrono::Utc>,
        end_threshold: chrono::DateTime<chrono::Utc>,
        activity: DepositAccountActivity,
    ) -> Result<Vec<DepositAccountId>, CoreDepositError> {
        let activity_str = activity.to_string();
        let rows = sqlx::query!(
            r#"
            SELECT da.deposit_account_id
            FROM deposit_account_activity da
            JOIN core_deposit_accounts c ON da.deposit_account_id = c.id
            WHERE da.last_activity_date >= $1
              AND da.last_activity_date < $2
              AND c.activity != $3
            "#,
            start_threshold,
            end_threshold,
            activity_str
        )
        .fetch_all(&self.pool)
        .await?;

        let account_ids: Vec<DepositAccountId> = rows
            .into_iter()
            .map(|row| DepositAccountId::from(row.deposit_account_id))
            .collect();

        Span::current().record("matched_count", account_ids.len());

        Ok(account_ids)
    }
}
