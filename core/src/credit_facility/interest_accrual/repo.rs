use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    data_export::Export,
    primitives::{CreditFacilityId, InterestAccrualId, InterestAccrualIdx},
};

use crate::credit_facility::error::CreditFacilityError;

use super::entity::*;

const BQ_TABLE_NAME: &str = "interest_accrual_events";

#[derive(Clone)]
pub(in crate::credit_facility) struct InterestAccrualRepo {
    _pool: PgPool,
    export: Export,
}

impl InterestAccrualRepo {
    pub fn new(pool: &PgPool, export: &Export) -> Self {
        Self {
            _pool: pool.clone(),
            export: export.clone(),
        }
    }

    pub async fn create_in_tx(
        &self,
        db: &mut Transaction<'_, Postgres>,
        new_interest_accrual: NewInterestAccrual,
    ) -> Result<InterestAccrual, CreditFacilityError> {
        sqlx::query!(
            r#"INSERT INTO interest_accruals (id, credit_facility_id, idx)
            VALUES ($1, $2, $3)"#,
            new_interest_accrual.id as InterestAccrualId,
            new_interest_accrual.facility_id as CreditFacilityId,
            new_interest_accrual.idx as InterestAccrualIdx,
        )
        .execute(&mut **db)
        .await?;
        let mut events = new_interest_accrual.initial_events();
        let n_events = events.persist(db).await?;
        self.export
            .export_last(db, BQ_TABLE_NAME, n_events, &events)
            .await?;
        Ok(InterestAccrual::try_from(events)?)
    }
}
