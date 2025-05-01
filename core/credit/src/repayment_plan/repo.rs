use sqlx::PgPool;
use uuid::Uuid;

use crate::primitives::CreditFacilityId;

use super::{error::*, CreditFacilityRepaymentPlan};

#[derive(Clone)]
pub struct RepaymentPlanRepo {
    pool: PgPool,
}

impl RepaymentPlanRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn begin(
        &self,
    ) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, CreditFacilityRepaymentPlanError> {
        Ok(self.pool.begin().await?)
    }

    pub async fn persist_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        credit_facility_id: CreditFacilityId,
        repayment_plan: CreditFacilityRepaymentPlan,
    ) -> Result<(), CreditFacilityRepaymentPlanError> {
        let json = serde_json::to_value(repayment_plan).expect("Could not serialize dashboard");
        let credit_facility_id: Uuid = credit_facility_id.into();
        sqlx::query!(
            r#"
            INSERT INTO core_credit_facility_repayment_plans (id, repayment_plan)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET repayment_plan = $2
            "#,
            credit_facility_id,
            json
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn load(
        &self,
        credit_facility_id: CreditFacilityId,
    ) -> Result<CreditFacilityRepaymentPlan, CreditFacilityRepaymentPlanError> {
        let credit_facility_id: Uuid = credit_facility_id.into();

        let row = sqlx::query!(
            "SELECT repayment_plan FROM core_credit_facility_repayment_plans WHERE id = $1",
            credit_facility_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let repayment_plan = if let Some(row) = row {
            serde_json::from_value(row.repayment_plan).expect("valid json")
        } else {
            CreditFacilityRepaymentPlan::default()
        };

        Ok(repayment_plan)
    }
}
