use sqlx::PgPool;

use crate::primitives::CreditFacilityId;

use super::{CreditFacilityHistory, error::*};

#[derive(Clone)]
pub struct HistoryRepo {
    pool: PgPool,
}

impl HistoryRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn begin(
        &self,
    ) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, CreditFacilityHistoryError> {
        Ok(self.pool.begin().await?)
    }

    #[tracing::instrument(
        name = "credit_facility_history.persist_in_tx",
        skip_all,
        err(level = "warn")
    )]
    pub async fn persist_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        credit_facility_proposal_id: impl Into<CreditFacilityId>,
        history: CreditFacilityHistory,
    ) -> Result<(), CreditFacilityHistoryError> {
        let id = credit_facility_proposal_id.into();
        let json = serde_json::to_value(history).expect("Could not serialize dashboard");
        sqlx::query!(
            r#"
            INSERT INTO core_credit_facility_histories (id, history)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET history = $2
            "#,
            id as CreditFacilityId,
            json
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    #[tracing::instrument(name = "credit_facility_history.load", skip_all, err(level = "warn"))]
    pub async fn load(
        &self,
        credit_facility_proposal_id: impl Into<CreditFacilityId>,
    ) -> Result<CreditFacilityHistory, CreditFacilityHistoryError> {
        let id = credit_facility_proposal_id.into();
        let row = sqlx::query!(
            "SELECT history FROM core_credit_facility_histories WHERE id = $1",
            id as CreditFacilityId,
        )
        .fetch_optional(&self.pool)
        .await?;

        let history = if let Some(row) = row {
            serde_json::from_value(row.history).expect("valid json")
        } else {
            CreditFacilityHistory::default()
        };

        Ok(history)
    }
}
