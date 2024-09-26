use sqlx::PgPool;

use crate::primitives::CreditFacilityTermsId;

use super::{error::CreditFacilityTermsError, TermValues, Terms};

#[derive(Clone)]
pub struct TermRepo {
    pool: PgPool,
}

impl TermRepo {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn update_default(
        &self,
        terms: TermValues,
    ) -> Result<Terms, CreditFacilityTermsError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"
             UPDATE credit_facility_terms
             SET current = FALSE
             WHERE current IS TRUE
            "#,
        )
        .execute(&mut *tx)
        .await?;

        let row = sqlx::query!(
            r#"
             INSERT INTO credit_facility_terms (current, values)
             VALUES (TRUE, $1)
             RETURNING id
            "#,
            serde_json::to_value(&terms).expect("should serialize term values"),
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(Terms {
            id: CreditFacilityTermsId::from(row.id),
            values: terms,
        })
    }

    pub async fn find_default(&self) -> Result<Terms, CreditFacilityTermsError> {
        let row = sqlx::query!(
            r#"
            SELECT id, values
            FROM credit_facility_terms
            WHERE current IS TRUE
            "#,
        )
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(row) => Ok(Terms {
                id: CreditFacilityTermsId::from(row.id),
                values: serde_json::from_value(row.values).expect("should deserialize term values"),
            }),
            Err(sqlx::Error::RowNotFound) => Err(CreditFacilityTermsError::TermsNotSet),
            Err(err) => Err(err.into()),
        }
    }
}
