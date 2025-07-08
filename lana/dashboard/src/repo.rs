use sqlx::PgPool;

use crate::{error::*, values::*};

const DASHBOARD_ID: uuid::Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000000");

#[derive(Clone)]
pub struct DashboardRepo {
    pool: PgPool,
}

impl DashboardRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn begin(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, DashboardError> {
        Ok(self.pool.begin().await?)
    }

    pub async fn persist_in_op(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        values: &DashboardValues,
    ) -> Result<(), DashboardError> {
        let values = serde_json::to_value(values).expect("Could not serialize dashboard");
        sqlx::query!(
            r#"
            INSERT INTO dashboards (id, dashboard_json)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE
            SET dashboard_json = $2, modified_at = NOW()
            "#,
            DASHBOARD_ID,
            values
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn load(&self) -> Result<DashboardValues, DashboardError> {
        let row = sqlx::query!(
            r#" 
            SELECT dashboard_json
            FROM dashboards
            WHERE id = $1
            "#,
            DASHBOARD_ID
        )
        .fetch_optional(&self.pool)
        .await?;
        let res = if let Some(row) = row {
            let values: DashboardValues = serde_json::from_value(row.dashboard_json)
                .expect("Could not de-serialize dashboard");
            values
        } else {
            DashboardValues::default()
        };
        Ok(res)
    }
}
